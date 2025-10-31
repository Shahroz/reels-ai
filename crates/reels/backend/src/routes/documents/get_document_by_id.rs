//! Retrieves a specific document entry by ID for the authenticated user.
//!
//! Uses the provided database pool and user claims to fetch the document entry.
//! Returns 200 with the entry if found, 404 if not found, or 500 on error.
//! Allows access to documents owned by the user or public documents.

use crate::auth::tokens::Claims;
use crate::db::documents::Document;
use crate::routes::documents::responses::DocumentResponseWithFavorite;
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

// Helper struct for query result to include ownership/share info
#[derive(sqlx::FromRow, Debug)]
struct DocumentWithAccessInfo {
    id: Uuid,
    user_id: Option<Uuid>, // Document's original owner/creator
    title: String,
    content: String,
    sources: Vec<String>,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    is_public: bool,
    is_task: bool,
    include_research: Option<String>,
    collection_id: Option<Uuid>,
    creator_email: Option<String>, // Email of the document's original owner/creator
    shared_access_level: Option<String>, // Access level from object_shares ('viewer', 'editor')
    is_favorite: Option<bool>, // Indicates if the document is a favorite of the authenticated user
}

#[utoipa::path(
    get,
    path = "/api/documents/{id}",
    tag = "Documents",
    params(
        ("id" = String, Path, description = "Document entry ID")
    ),
    responses(
        (status = 200, description = "Document entry found", body = DocumentResponseWithFavorite),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[get("{id}")]
pub async fn get_document_by_id(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let document_id = path.into_inner();

    // Fetch active organization IDs for the authenticated user
    let org_ids_result = sqlx::query_scalar!(
        "SELECT organization_id FROM organization_members WHERE user_id = $1 AND status = 'active'",
        authenticated_user_id
    )
    .fetch_all(&**pool)
    .await;

    let org_ids = match org_ids_result {
        Ok(ids) => ids,
        Err(e) => {
            log::error!(
                "Failed to fetch organization IDs for user {authenticated_user_id}: {e}"
            );
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch organization memberships".into(),
            });
        }
    };

    // Refactored query using sqlx::query_as!
    let result = sqlx::query_as!(
        DocumentWithAccessInfo,
        r#"
        WITH UserOrgShares AS (
            -- Direct user shares for this document
            SELECT access_level
            FROM object_shares
            WHERE object_id = $1 -- document_id
              AND entity_id = $2 -- authenticated_user_id
              AND entity_type = 'user' -- Assumes 'object_share_entity_type' enum in DB
              AND object_type = 'document'
        UNION ALL
            -- Organization shares for user's orgs for this document
            SELECT os.access_level
            FROM object_shares os
            WHERE os.object_id = $1 -- document_id
              AND os.entity_id = ANY($3::UUID[]) -- org_ids (Vec<Uuid>)
              AND os.entity_type = 'organization' -- Assumes 'object_share_entity_type' enum in DB
              AND os.object_type = 'document'
        ),
        CollectionShares AS (
            -- User shares for the collection containing this document
            SELECT os.access_level
            FROM object_shares os
            INNER JOIN documents d ON d.collection_id = os.object_id
            WHERE d.id = $1 -- document_id
              AND os.entity_id = $2 -- authenticated_user_id
              AND os.entity_type = 'user'
              AND os.object_type = 'collection'
        UNION ALL
            -- Organization shares for the collection containing this document
            SELECT os.access_level
            FROM object_shares os
            INNER JOIN documents d ON d.collection_id = os.object_id
            WHERE d.id = $1 -- document_id
              AND os.entity_id = ANY($3::UUID[]) -- org_ids
              AND os.entity_type = 'organization'
              AND os.object_type = 'collection'
        ),
        AllShares AS (
            SELECT access_level FROM UserOrgShares
            UNION ALL
            SELECT access_level FROM CollectionShares
        ),
        RankedShares AS (
            SELECT
                access_level,
                ROW_NUMBER() OVER (ORDER BY
                    CASE access_level
                        WHEN 'editor' THEN 1 -- Assumes 'access_level_enum' in DB for access_level column
                        WHEN 'viewer' THEN 2
                        ELSE 3
                    END
                ) as rn
            FROM AllShares
        )
        SELECT
            d.id,
            d.user_id, -- Mapped to Option<Uuid> in DocumentWithAccessInfo
            d.title,
            d.content,
            d.sources, -- Mapped to Vec<String>
            d.status,
            d.created_at,
            d.updated_at,
            d.is_public,
            d.is_task,
            d.include_research::TEXT,
            d.collection_id,
            (SELECT email FROM users WHERE id = d.user_id) AS creator_email, -- Mapped to Option<String>
            (SELECT rs.access_level::TEXT FROM RankedShares rs WHERE rs.rn = 1) AS shared_access_level, -- Mapped to Option<String>
            COALESCE((SELECT EXISTS(SELECT 1 FROM user_favorites WHERE user_id = $2 AND entity_id = d.id AND entity_type = 'document')), false) AS is_favorite
        FROM
            documents d
        WHERE
            d.id = $1 -- document_id
        "#,
        document_id,           // $1
        authenticated_user_id, // $2
        &org_ids               // $3: Pass as reference for ANY($N::UUID[])
    )
    .fetch_optional(&**pool)
    .await;

    match result {
        Ok(Some(doc_info)) => {
            let current_user_access_level = if Some(authenticated_user_id) == doc_info.user_id {
                Some("owner".to_string())
            } else {
                doc_info.shared_access_level
            };

            if current_user_access_level.is_none() && !doc_info.is_public {
                return HttpResponse::NotFound().json(ErrorResponse {
                    error: "Document not found or access denied".into(),
                });
            }
            
            let final_access_level = if current_user_access_level.is_none() && doc_info.is_public {
                None 
            } else {
                current_user_access_level
            };

            let document = Document {
                id: doc_info.id,
                user_id: doc_info.user_id,
                title: doc_info.title,
                content: doc_info.content,
                sources: doc_info.sources,
                status: doc_info.status,
                created_at: doc_info.created_at,
                updated_at: doc_info.updated_at,
                is_public: doc_info.is_public,
                is_task: doc_info.is_task,
                include_research: doc_info.include_research.map(|s| {
                    s.parse()
                        .unwrap_or(crate::db::document_research_usage::DocumentResearchUsage::TaskDependent)
                }),
                collection_id: doc_info.collection_id,
            };

            let response = DocumentResponseWithFavorite {
                document,
                creator_email: doc_info.creator_email,
                current_user_access_level: final_access_level,
                is_favorite: doc_info.is_favorite.unwrap_or(false),
            };

            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Document not found".into(),
        }),
        Err(e) => {
            log::error!(
                "Error retrieving document {document_id} for user {authenticated_user_id}: {e}"
            );
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve document".into(),
            })
        }
    }
}
