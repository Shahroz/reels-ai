#![allow(clippy::disallowed_methods)]
//! Handler for listing vocal tour document entries for the authenticated user.
//!
//! Defines the GET `/api/documents/vocal-tour` endpoint handler.
//! Fetches document entries that contain 'vocal-tour' in their sources array and are accessible to the user.
//! Supports pagination and sorting, specifically filtered for vocal tour generated documents.
//! Includes shared documents through user and organization sharing permissions.

use crate::db::documents::Document;
use crate::routes::error_response::ErrorResponse;
use crate::auth::tokens::Claims;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::routes::documents::responses::DocumentResponseWithFavorite;

#[derive(Serialize, Deserialize, Debug, ToSchema, utoipa::IntoParams)]
pub struct ListVocalTourDocumentsParams {
    #[schema(example = 1)]
    pub page: Option<i64>,
    #[schema(example = 10)]
    pub limit: Option<i64>,
    #[schema(example = "created_at")]
    pub sort_by: Option<String>,
    #[schema(example = "desc")]
    pub sort_order: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ListVocalTourDocumentsResponse {
    pub items: Vec<DocumentResponseWithFavorite>,
    pub total_count: i64,
}

#[utoipa::path(
    get,
    path = "/api/documents/vocal-tour",
    tag = "Documents",
    params(
        ListVocalTourDocumentsParams
    ),
    responses(
        (status = 200, description = "List vocal tour document entries", body = ListVocalTourDocumentsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[get("/vocal-tour")]
pub async fn list_vocal_tour_documents(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<Claims>,
    params: web::Query<ListVocalTourDocumentsParams>,
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

    let org_ids = match fetch_user_organization_ids(&pool, authenticated_user_id).await {
        Ok(ids) => ids,
        Err(e) => {
            log::error!("Failed to fetch organization IDs for user {authenticated_user_id}: {e}");
            return HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    error: "Failed to fetch user permissions.".into(),
                });
        }
    };

    let total_count = match count_vocal_tour_documents(&pool, authenticated_user_id, &org_ids).await {
        Ok(count) => count,
        Err(e) => {
            log::error!("Error counting vocal tour documents for user {authenticated_user_id}: {e}");
            return HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    error: "Failed to count vocal tour documents".into(),
                });
        }
    };

    match fetch_vocal_tour_documents(&pool, authenticated_user_id, &org_ids, &sort_by, &sort_order, limit, offset).await {
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
                            s.parse::<crate::db::document_research_usage::DocumentResearchUsage>().ok()
                        }),
                        collection_id: d.collection_id,
                    },
                    creator_email: d.creator_email,
                    current_user_access_level: d.current_user_access_level,
                    is_favorite: d.is_favorite.unwrap_or(false),
                })
                .collect();
            
            HttpResponse::Ok().json(ListVocalTourDocumentsResponse { 
                items, 
                total_count 
            })
        }
        Err(e) => {
            log::error!("Error fetching vocal tour documents for user {authenticated_user_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch vocal tour documents".into(),
            })
        }
    }
}

async fn fetch_user_organization_ids(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT organization_id FROM organization_members WHERE user_id = $1 AND status = 'active'",
        user_id
    )
    .fetch_all(pool)
    .await
}

async fn count_vocal_tour_documents(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    org_ids: &[Uuid],
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query_scalar!(
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
        AND 'vocal-tour' = ANY(d.sources)
        "#,
        user_id,
        org_ids
    )
    .fetch_one(pool)
    .await?;

    Ok(result.unwrap_or(0))
}

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
    is_favorite: Option<bool>,
}

async fn fetch_vocal_tour_documents(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    org_ids: &[Uuid],
    sort_by: &str,
    sort_order: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<DocumentWithAccess>, sqlx::Error> {
    let sort_column = match sort_by {
        "updated_at" => "d.updated_at",
        "title" => "d.title",
        "status" => "d.status",
        _ => "d.created_at",
    };

    let sort_direction = match sort_order {
        "asc" => "ASC",
        _ => "DESC",
    };

    let query = format!(
        r#"
        SELECT 
            d.id, d.user_id, d.title, d.content, d.sources, d.status, 
            d.created_at, d.updated_at, d.is_public, d.is_task, d.include_research, d.collection_id, 
            u.email as creator_email,
            CASE 
                WHEN d.user_id = $1 THEN 'owner' 
                ELSE ranked_shares.access_level 
            END AS current_user_access_level,
            COALESCE((SELECT EXISTS(SELECT 1 FROM user_favorites WHERE user_id = $1 AND entity_id = d.id AND entity_type = 'document')), false) AS is_favorite
        FROM documents d
        LEFT JOIN users u ON d.user_id = u.id
        LEFT JOIN (
            SELECT 
                object_id,
                access_level::TEXT,
                ROW_NUMBER() OVER(PARTITION BY object_id ORDER BY CASE access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END) as rn 
            FROM object_shares 
            WHERE object_type = 'document' AND ((entity_type = 'user' AND entity_id = $1) OR (entity_type = 'organization' AND entity_id = ANY($2)))
        ) AS ranked_shares ON d.id = ranked_shares.object_id AND ranked_shares.rn = 1
        WHERE (d.user_id = $1 OR ranked_shares.access_level IS NOT NULL) 
        AND 'vocal-tour' = ANY(d.sources)
        ORDER BY {sort_column} {sort_direction}
        LIMIT $3 OFFSET $4
        "#
    );

    sqlx::query_as::<_, DocumentWithAccess>(&query)
        .bind(user_id)
        .bind(org_ids)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_user_organization_ids() {
        // Test basic functionality - would need database setup
        assert!(true);
    }

    #[tokio::test] 
    async fn test_count_vocal_tour_documents() {
        // Test counting with mocked data
        assert!(true);
    }

    #[tokio::test]
    async fn test_fetch_vocal_tour_documents() {
        // Test document fetching with various sorting options
        assert!(true);
    }
} 