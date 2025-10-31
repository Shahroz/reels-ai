#![allow(clippy::disallowed_methods)]
//! Handler for listing document entries for the authenticated user, including shared documents.
//!
//! Defines the GET `/api/documents` endpoint handler.
//! Fetches document entries that are either owned by the user, shared directly with the user,
//! or shared with an organization the user is a member of.
//! Supports pagination, sorting, and searching.

// Retain specific imports for utoipa::path `body` short names, as per zenide.md
use crate::db::documents::Document;
use crate::routes::error_response::ErrorResponse;
use crate::auth::tokens::Claims;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::routes::documents::responses::DocumentResponseWithFavorite;
use sqlx_conditional_queries::conditional_query_as;

#[derive(Serialize, Deserialize, Debug, ToSchema, utoipa::IntoParams)]
pub struct ListDocumentsParams {
    #[schema(example = 1)]
    pub page: Option<i64>,
    #[schema(example = 10)]
    pub limit: Option<i64>,
    #[schema(example = "created_at")]
    pub sort_by: Option<String>,
    #[schema(example = "desc")]
    pub sort_order: Option<String>,
    #[schema(example = "market analysis")]
    pub search: Option<String>,
    #[schema(example = false)]
    pub is_public: Option<bool>,
    #[schema(example = false, nullable = true)]
    pub is_task: Option<bool>,
   #[schema(example = true, nullable = true)]
   pub is_favorite: Option<bool>,
   #[schema(example = "Always", nullable = true)]
   pub include_research: Option<String>,
   #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", nullable = true)]
   pub collection_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ListDocumentsResponse {
    pub items: Vec<DocumentResponseWithFavorite>,
    pub total_count: i64,
}

#[utoipa::path(
    get,
    path = "/api/documents",
    tag = "Documents",
    params(
        ListDocumentsParams
    ),
    responses(
        (status = 200, description = "List document entries", body = ListDocumentsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[get("")]
pub async fn list_documents(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<Claims>,
    params: web::Query<ListDocumentsParams>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).max(1);
    let offset = (page - 1) * limit;

    let sort_by = params
        .sort_by
        .clone()
        .filter(|s| ["created_at", "updated_at", "title", "status"].contains(&s.as_str()))
        .unwrap_or_else(|| "created_at".to_string());
    let sort_order = params
        .sort_order
        .clone()
        .filter(|s| s == "asc" || s == "desc")
        .unwrap_or_else(|| "desc".to_string());
    let search_pattern = params
        .search
        .clone()
        .map(|s| format!("%{s}%"))
        .unwrap_or_else(|| "%".to_string());
   let is_public_filter = params.is_public.unwrap_or(false);
   let is_task_filter = params.is_task; // Option<bool>
   let is_favorite_filter = params.is_favorite; // Option<bool>
   let include_research_filter = params.include_research.clone(); // Option<String>
   let collection_id_filter = params.collection_id; // Option<Uuid>

    #[derive(sqlx::FromRow, Debug)]
    struct DocumentWithAccess {
        id: Uuid,
        user_id: Option<Uuid>,
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
        creator_email: Option<String>,
        current_user_access_level: Option<String>,
        is_favorite: Option<bool>, // Changed from bool to Option<bool> to handle potential NULL values
    }

   if is_public_filter {
        // Build and execute query for public documents
        let public_count_result = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM documents d WHERE d.is_public = true AND (d.title ILIKE $1 OR d.content ILIKE $1) AND ($2::BOOLEAN IS NULL OR d.is_task = $2) AND ($3::BOOLEAN IS NULL OR EXISTS(SELECT 1 FROM user_favorites WHERE user_id = $4 AND entity_id = d.id AND entity_type = 'document') = $3) AND ($5::TEXT IS NULL OR d.include_research = $5) AND ($6::UUID IS NULL OR d.collection_id = $6)"#,
            &search_pattern,
            is_task_filter,
            is_favorite_filter,
            authenticated_user_id,
            include_research_filter,
            collection_id_filter
        )
        .fetch_one(&**pool)
        .await;

        let total_count = match public_count_result {
            Ok(Some(count)) => count,
            Ok(None) => 0,
            Err(e) => {
                log::error!("Error counting public documents: {e}");
                return HttpResponse::InternalServerError()
                    .json(ErrorResponse {
                        error: "Failed to count public documents".into(),
                    });
            }
        };

       let items_result = conditional_query_as!(
            DocumentWithAccess,
            r#"
            SELECT id, user_id, title, content, sources, status, created_at, updated_at, is_public, is_task, include_research, collection_id, NULL as creator_email, NULL as current_user_access_level,
                COALESCE((SELECT EXISTS(SELECT 1 FROM user_favorites WHERE user_id = {authenticated_user_id} AND entity_id = id AND entity_type = 'document')), false) AS is_favorite
             FROM documents WHERE is_public = true AND (title ILIKE {search_pattern} OR content ILIKE {search_pattern}) AND ({is_task_filter}::BOOLEAN IS NULL OR is_task = {is_task_filter}) AND ({is_favorite_filter}::BOOLEAN IS NULL OR EXISTS(SELECT 1 FROM user_favorites WHERE user_id = {authenticated_user_id} AND entity_id = id AND entity_type = 'document') = {is_favorite_filter}) AND ({include_research_filter}::TEXT IS NULL OR include_research = {include_research_filter}) AND ({collection_id_filter}::UUID IS NULL OR collection_id = {collection_id_filter})
             ORDER BY {#sort_by_sql_literal} {#sort_order_sql_literal}
             LIMIT {limit} OFFSET {offset}
            "#,
            #search_pattern = match &search_pattern { _ => "{search_pattern}" },
            #is_task_filter = match &is_task_filter { _ => "{is_task_filter}" },
            #is_favorite_filter = match &is_favorite_filter { _ => "{is_favorite_filter}" },
            #include_research_filter = match &include_research_filter { _ => "{include_research_filter}" },
            #collection_id_filter = match &collection_id_filter { _ => "{collection_id_filter}" },
            #limit = match &limit { _ => "{limit}" },
            #offset = match &offset { _ => "{offset}" },
            #sort_by_sql_literal = match sort_by.as_str() {
                "updated_at" => "updated_at",
                "title" => "title",
                "status" => "status",
                _ => "created_at",
            },
            #sort_order_sql_literal = match sort_order.as_str() {
                "asc" => "ASC",
                "desc" => "DESC",
                _ => "DESC",
            },
            #authenticated_user_id = match &authenticated_user_id { _ => "{authenticated_user_id}" }
        )
            .fetch_all(&**pool)
            .await;

        return match items_result {
            Ok(docs) => {
                let items = docs
                    .into_iter()
                    .map(|d| DocumentResponseWithFavorite {
                        document: Document {
                            id: d.id,
                            user_id: d.user_id,
                            title: d.title,
                            content: d.content,
                            sources: d.sources,
                            status: d.status,
                            created_at: d.created_at,
                            updated_at: d.updated_at,
                            is_public: d.is_public,
                            is_task: d.is_task,
                            include_research: d.include_research.and_then(|s| {
                                s.parse::<crate::db::document_research_usage::DocumentResearchUsage>()
                                    .map_err(|e| log::warn!("Failed to parse include_research '{s}': {e}"))
                                    .ok()
                            }),
                            collection_id: d.collection_id,
                        },
                        creator_email: d.creator_email,
                        current_user_access_level: d.current_user_access_level,
                        is_favorite: d.is_favorite.unwrap_or(false),
                    })
                    .collect();
                HttpResponse::Ok().json(ListDocumentsResponse { items, total_count })
            }
            Err(e) => {
                log::error!("Error fetching public documents: {e}");
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to fetch public documents".into(),
                })
            }
        };
    }

    let org_ids: Vec<Uuid> = match sqlx::query_scalar!(
        "SELECT organization_id FROM organization_members WHERE user_id = $1 AND status = 'active'",
        authenticated_user_id
    )
    .fetch_all(&**pool)
    .await
    {
        Ok(ids) => ids,
        Err(e) => {
            log::error!(
                "Failed to fetch organization IDs for user {authenticated_user_id}: {e}"
            );
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch user permissions.".into(),
            });
        }
    };

    let total_count_result = sqlx::query_scalar!(
        r#"
       SELECT COUNT(DISTINCT d.id)
       FROM documents d
        LEFT JOIN object_shares os_user ON d.id = os_user.object_id
            AND os_user.object_type = 'document'
            AND os_user.entity_type = 'user'
            AND os_user.entity_id = $1
        LEFT JOIN object_shares os_org ON d.id = os_org.object_id
            AND os_org.object_type = 'document'
            AND os_org.entity_type = 'organization'
            AND os_org.entity_id = ANY($2)
        WHERE (d.user_id = $1 OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL)
        AND (d.title ILIKE $3 OR d.content ILIKE $3)
        AND ($4::BOOLEAN IS NULL OR d.is_task = $4)
        AND ($5::BOOLEAN IS NULL OR EXISTS(SELECT 1 FROM user_favorites WHERE user_id = $1 AND entity_id = d.id AND entity_type = 'document') = $5) AND ($6::TEXT IS NULL OR d.include_research = $6) AND ($7::UUID IS NULL OR d.collection_id = $7)
        "#,
        authenticated_user_id,
        &org_ids,
        &search_pattern,
        is_task_filter,
        is_favorite_filter,
        include_research_filter,
        collection_id_filter
    )
    .fetch_one(&**pool)
    .await;

    let total_count = match total_count_result {
        Ok(Some(count)) => count,
        Ok(None) => 0,
        Err(e) => {
            log::error!(
                "Error counting documents for user {authenticated_user_id}: {e}"
            );
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to count documents".into(),
            });
        }
    };

    let org_ids_slice = org_ids.as_slice();
    let items_result = conditional_query_as!(
        DocumentWithAccess,
        r#"
        SELECT 
            d.id, d.user_id, d.title, d.content, d.sources, d.status, 
            d.created_at, d.updated_at, d.is_public, d.is_task, d.include_research, d.collection_id, 
            u.email as creator_email,
            CASE 
                WHEN d.user_id = {authenticated_user_id} THEN 'owner' 
                WHEN ranked_shares.access_level IS NOT NULL THEN ranked_shares.access_level
                ELSE ranked_collection_shares.access_level 
            END AS current_user_access_level,
            COALESCE((SELECT EXISTS(SELECT 1 FROM user_favorites WHERE user_id = {authenticated_user_id} AND entity_id = d.id AND entity_type = 'document')), false) AS is_favorite
        FROM documents d
        LEFT JOIN users u ON d.user_id = u.id
        LEFT JOIN (
            SELECT 
                object_id,
                access_level::TEXT,
                ROW_NUMBER() OVER(PARTITION BY object_id ORDER BY CASE access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END) as rn 
            FROM object_shares 
            WHERE object_type = 'document' AND ((entity_type = 'user' AND entity_id = {authenticated_user_id}) OR (entity_type = 'organization' AND entity_id = ANY({org_ids_slice})))
        ) AS ranked_shares ON d.id = ranked_shares.object_id AND ranked_shares.rn = 1 
        LEFT JOIN (
            SELECT 
                object_id as collection_id,
                access_level::TEXT,
                ROW_NUMBER() OVER(PARTITION BY object_id ORDER BY CASE access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END) as rn 
            FROM object_shares 
            WHERE object_type = 'collection' AND ((entity_type = 'user' AND entity_id = {authenticated_user_id}) OR (entity_type = 'organization' AND entity_id = ANY({org_ids_slice})))
        ) AS ranked_collection_shares ON d.collection_id = ranked_collection_shares.collection_id AND ranked_collection_shares.rn = 1 
        WHERE (d.user_id = {authenticated_user_id} OR ranked_shares.access_level IS NOT NULL OR ranked_collection_shares.access_level IS NOT NULL) AND (d.title ILIKE {search_pattern} OR d.content ILIKE {search_pattern}) AND ({is_task_filter}::BOOLEAN IS NULL OR d.is_task = {is_task_filter}) AND ({is_favorite_filter}::BOOLEAN IS NULL OR EXISTS(SELECT 1 FROM user_favorites WHERE user_id = {authenticated_user_id} AND entity_id = d.id AND entity_type = 'document') = {is_favorite_filter}) AND ({include_research_filter}::TEXT IS NULL OR d.include_research = {include_research_filter}) AND ({collection_id_filter}::UUID IS NULL OR d.collection_id = {collection_id_filter})
        ORDER BY {#sort_by_sql_literal} {#sort_order_sql_literal}
        LIMIT {limit} OFFSET {offset}
        "#,
        #authenticated_user_id = match &authenticated_user_id { _ => "{authenticated_user_id}" },
        #org_ids_slice = match &org_ids_slice { _ => "{org_ids_slice}" },
        #search_pattern = match &search_pattern { _ => "{search_pattern}" },
        #is_task_filter = match &is_task_filter { _ => "{is_task_filter}" },
        #is_favorite_filter = match &is_favorite_filter { _ => "{is_favorite_filter}" },
        #include_research_filter = match &include_research_filter { _ => "{include_research_filter}" },
        #collection_id_filter = match &collection_id_filter { _ => "{collection_id_filter}" },
        #limit = match &limit { _ => "{limit}" },
        #offset = match &offset { _ => "{offset}" },
        #sort_by_sql_literal = match sort_by.as_str() {
            "updated_at" => "d.updated_at",
            "title" => "d.title",
            "status" => "d.status",
            _ => "d.created_at",
        },
        #sort_order_sql_literal = match sort_order.as_str() {
            "asc" => "ASC",
            "desc" => "DESC",
            _ => "DESC",
        }
    )
    .fetch_all(&**pool).await;

    match items_result {
        Ok(docs) => {
            let items = docs
                .into_iter()
                .map(|d| DocumentResponseWithFavorite {
                    document: Document {
                        id: d.id,
                        user_id: d.user_id,
                        title: d.title,
                        content: d.content,
                        sources: d.sources,
                        status: d.status,
                        created_at: d.created_at,
                        updated_at: d.updated_at,
                        is_public: d.is_public,
                        is_task: d.is_task,
                        include_research: d.include_research.and_then(|s| {
                            s.parse::<crate::db::document_research_usage::DocumentResearchUsage>()
                                .map_err(|e| log::warn!("Failed to parse include_research '{s}': {e}"))
                                .ok()
                        }),
                        collection_id: d.collection_id,
                    },
                    creator_email: d.creator_email,
                    current_user_access_level: d.current_user_access_level,
                    is_favorite: d.is_favorite.unwrap_or(false),
                })
                .collect();
            HttpResponse::Ok().json(ListDocumentsResponse { items, total_count })
        }
        Err(e) => {
            log::error!(
                "Error fetching documents for user {authenticated_user_id}: {e}"
            );
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch documents".into(),
            })
        }
    }
}
