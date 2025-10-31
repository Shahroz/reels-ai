//! Search route module.
//! Provides the global search endpoint.
use actix_web::{get, web, HttpResponse};
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use tracing::instrument;
use crate::db::styles::Style;
use crate::db::assets::Asset;
use crate::db::creatives::Creative;
use crate::db::documents::Document; // Changed from Research to Document

#[derive(Deserialize, ToSchema)]
pub struct SearchQuery {
    /// Search query parameter
    pub q: String,
}

#[derive(Serialize, ToSchema)]
pub struct SearchResult {
    pub styles: Vec<Style>,
    pub assets: Vec<Asset>,
    pub creatives: Vec<Creative>,
    pub documents: Vec<Document>, // Changed from research to documents
}

#[utoipa::path(
    get,
    path = "/api/search",
    params(
        ("q" = String, Query, description = "Search query")
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResult),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/api/search")]
#[instrument(skip(pool, query))]
pub async fn global_search(
    pool: web::Data<PgPool>,
    web::Query(query): web::Query<SearchQuery>,
) -> HttpResponse {
    let pattern = format!("%{}%", query.q);
    let styles_res = sqlx::query_as!(Style, 
        r#"
        SELECT 
            s.id, s.user_id, s.name, s.html_url, s.screenshot_url, s.is_public,
            s.created_at, s.updated_at
        FROM styles s
        WHERE s.name ILIKE $1
        "#,
         &pattern
    )
        .fetch_all(pool.get_ref())
        .await;
    let assets_res = sqlx::query_as!(Asset, "SELECT id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public FROM assets WHERE name ILIKE $1 AND is_public = FALSE", &pattern)
        .fetch_all(pool.get_ref())
        .await;
    // Creatives table does not have a 'name' column. Searching by 'html_url' or 'id' as a placeholder.
    // Explicitly list columns as per Creative struct observed in get_creative_by_id.rs
    let creatives_res = sqlx::query_as!(
            Creative, 
            r#"SELECT 
                id, name, collection_id, creative_format_id, style_id, document_ids, asset_ids, 
                html_url, draft_url, bundle_id, screenshot_url, is_published, publish_url, created_at, updated_at
            FROM creatives 
            WHERE html_url ILIKE $1 OR CAST(id AS TEXT) ILIKE $1
            "#,
            &pattern
        )
        .fetch_all(pool.get_ref())
        .await;
    let documents_res = sqlx::query_as!(
            Document,
            r#"SELECT id, user_id, title, content, sources, status, created_at, updated_at, is_public,
            is_task, include_research AS "include_research: _", collection_id
            FROM documents WHERE title ILIKE $1 OR content ILIKE $1"#,
            &pattern
        )
        .fetch_all(pool.get_ref())
        .await;

    match (styles_res, assets_res, creatives_res, documents_res) {
        (Ok(styles), Ok(assets), Ok(creatives), Ok(documents)) => {
            let result = SearchResult { styles, assets, creatives, documents };
            HttpResponse::Ok().json(result)
        },
        _ => HttpResponse::InternalServerError().finish(),
    }
}