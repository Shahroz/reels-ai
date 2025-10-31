//! Retrieves a specific document entry by name (title) for the authenticated user.
//!
//! Uses the provided database pool and user claims to fetch the document entry by title.
//! Returns 200 with the entry if found, 404 if not found, or 500 on error.
//! Allows access to documents owned by the user or public documents.
//! The document name in the URL path should be URL encoded.

use crate::auth::tokens::Claims;
use crate::db::documents::Document;
use crate::routes::documents::responses::DocumentResponseWithFavorite;
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use urlencoding;

// Helper struct for query result to include ownership/share info
#[derive(sqlx::FromRow, Debug)]
struct DocumentWithAccessInfo {
    id: uuid::Uuid,
    user_id: Option<uuid::Uuid>, // Document's original owner/creator
    title: String,
    content: String,
    sources: Vec<String>,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    is_public: bool,
    is_task: bool,
    include_research: Option<String>,
    collection_id: Option<uuid::Uuid>,
    creator_email: Option<String>, // Email of the document's original owner/creator
    shared_access_level: Option<String>, // Access level from object_shares ('viewer', 'editor')
    is_favorite: Option<bool>, // Indicates if the document is a favorite of the authenticated user
}

#[utoipa::path(
    get,
    path = "/api/documents/by-name/{name}",
    tag = "Documents",
    params(
        ("name" = String, Path, description = "Document title (URL encoded)")
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
#[get("/by-name/{name}")]
pub async fn get_document_by_name(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let raw_document_name = path.into_inner();
    
    // Decode the URL-encoded document name
    let document_name = match urlencoding::decode(&raw_document_name) {
        Ok(decoded) => decoded.into_owned(),
        Err(e) => {
            log::error!("Failed to decode document name '{raw_document_name}': {e}");
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid document name encoding.".into(),
            });
        }
    };

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

    // Combined query: search by title with proper access control
    let result = sqlx::query_as!(
        DocumentWithAccessInfo,
        r#"
        WITH UserOrgShares AS (
            -- Direct user shares
            SELECT object_id, access_level
            FROM object_shares
            WHERE entity_id = $1 -- authenticated_user_id
              AND entity_type = 'user'
              AND object_type = 'document'
        UNION ALL
            -- Organization shares for user's orgs
            SELECT os.object_id, os.access_level
            FROM object_shares os
            WHERE os.entity_id = ANY($3::UUID[]) -- org_ids
              AND os.entity_type = 'organization'
              AND os.object_type = 'document'
        ),
        CollectionShares AS (
            -- User shares for collections
            SELECT d.id as object_id, os.access_level
            FROM object_shares os
            INNER JOIN documents d ON d.collection_id = os.object_id
            WHERE os.entity_id = $1 -- authenticated_user_id
              AND os.entity_type = 'user'
              AND os.object_type = 'collection'
        UNION ALL
            -- Organization shares for collections
            SELECT d.id as object_id, os.access_level
            FROM object_shares os
            INNER JOIN documents d ON d.collection_id = os.object_id
            WHERE os.entity_id = ANY($3::UUID[]) -- org_ids
              AND os.entity_type = 'organization'
              AND os.object_type = 'collection'
        ),
        AllShares AS (
            SELECT object_id, access_level FROM UserOrgShares
            UNION ALL
            SELECT object_id, access_level FROM CollectionShares
        ),
        RankedShares AS (
            SELECT
                object_id,
                access_level,
                ROW_NUMBER() OVER (PARTITION BY object_id ORDER BY
                    CASE access_level
                        WHEN 'editor' THEN 1
                        WHEN 'viewer' THEN 2
                        ELSE 3
                    END
                ) as rn
            FROM AllShares
        )
        SELECT
            d.id,
            d.user_id,
            d.title,
            d.content,
            d.sources,
            d.status,
            d.created_at,
            d.updated_at,
            d.is_public,
            d.is_task,
            d.include_research::TEXT,
            d.collection_id,
            (SELECT email FROM users WHERE id = d.user_id) AS creator_email,
            rs.access_level::TEXT AS shared_access_level,
            COALESCE((SELECT EXISTS(SELECT 1 FROM user_favorites WHERE user_id = $1 AND entity_id = d.id AND entity_type = 'document')), false) AS is_favorite
        FROM
            documents d
        LEFT JOIN RankedShares rs ON d.id = rs.object_id AND rs.rn = 1
        WHERE
            d.title = $2 -- document_name
            AND (
                d.user_id = $1 -- User owns the document
                OR d.is_public = true -- Document is public
                OR rs.access_level IS NOT NULL -- User has share access
            )
        ORDER BY 
            CASE 
                WHEN d.user_id = $1 THEN 1 -- Prioritize user's own documents
                WHEN d.is_public = true THEN 2 -- Then public documents
                ELSE 3 -- Then shared documents
            END
        LIMIT 1
        "#,
        authenticated_user_id, // $1
        document_name,         // $2
        &org_ids               // $3
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
                "Error retrieving document with title '{document_name}' for user {authenticated_user_id}: {e}"
            );
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve document".into(),
            })
        }
    }
} 