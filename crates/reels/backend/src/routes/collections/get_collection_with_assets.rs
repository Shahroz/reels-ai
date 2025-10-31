//! Handler for fetching a collection with its associated assets.
//!
//! Defines the `get_collection_with_assets` HTTP handler under `/api/collections/{collection_id}/assets`.
//! This handler validates ownership and returns collection details with paginated assets.
//! Follows the project's Rust coding standards with fully qualified paths.

use actix_web::{get, web, HttpResponse, Responder};
use tracing::instrument;

use crate::auth::tokens::Claims;

#[derive(serde::Deserialize, utoipa::IntoParams, std::fmt::Debug)]
pub struct GetCollectionAssetsParams {
    /// Search pattern for asset names (optional)
    pub search: std::option::Option<std::string::String>,
    /// Field to sort by: name, type, created_at, updated_at (default: created_at)
    pub sort_by: std::option::Option<std::string::String>,
    /// Sort order: asc or desc (default: desc)
    pub sort_order: std::option::Option<std::string::String>,
    /// Page number (default: 1)
    pub page: std::option::Option<i64>,
    /// Items per page (default: 20, max: 100)
    pub limit: std::option::Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/collections/{collection_id}/assets",
    tag = "Collections",
    params(
        ("collection_id" = uuid::Uuid, Path, description = "Collection ID"),
        GetCollectionAssetsParams
    ),
    responses(
        (status = 200, description = "Collection with assets retrieved successfully", body = crate::queries::collections::get_collection_with_assets::CollectionWithAssets),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden: User does not own the collection"),
        (status = 404, description = "Collection not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/{collection_id}/assets")]
#[instrument(name = "get_collection_with_assets", skip(pool, claims))]
pub async fn get_collection_with_assets(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<uuid::Uuid>,
    params: web::Query<GetCollectionAssetsParams>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let collection_id = path.into_inner();
    let user_id = claims.user_id;

    // Validate and set default parameters
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(20).min(100).max(1); // Default 20, max 100
    let offset = (page - 1) * limit;
    
    let sort_by = params
        .sort_by
        .as_ref()
        .filter(|s| ["name", "type", "created_at", "updated_at"].contains(&s.as_str()))
        .map(|s| s.as_str())
        .unwrap_or("created_at");
        
    let sort_order = params
        .sort_order
        .as_ref()
        .filter(|s| s.as_str() == "asc" || s.as_str() == "desc")
        .map(|s| s.as_str())
        .unwrap_or("desc");
        
    let search_pattern = params
        .search
        .as_ref()
        .map(|s| std::format!("%{s}%"))
        .unwrap_or_else(|| "%".to_string());

    // Query the collection with assets
    match crate::queries::collections::get_collection_with_assets::get_collection_with_assets(
        &pool,
        collection_id,
        user_id,
        &search_pattern,
        sort_by,
        sort_order,
        limit,
        offset,
    ).await {
        Ok(Some(collection_with_assets)) => {
            tracing::info!(
                "Retrieved collection {} with {} assets for user {}",
                collection_id,
                collection_with_assets.assets.len(),
                user_id
            );
            HttpResponse::Ok().json(collection_with_assets)
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Collection not found or you don't have permission to access it"
            }))
        }
        Err(e) => {
            tracing::error!("Database error while fetching collection with assets: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch collection with assets"
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_collection_assets_params_deserialize() {
        // Test basic parameter parsing
        let params = GetCollectionAssetsParams {
            search: Some("test".to_string()),
            sort_by: Some("name".to_string()),
            sort_order: Some("asc".to_string()),
            page: Some(2),
            limit: Some(50),
        };
        
        assert_eq!(params.search, Some("test".to_string()));
        assert_eq!(params.sort_by, Some("name".to_string()));
        assert_eq!(params.sort_order, Some("asc".to_string()));
        assert_eq!(params.page, Some(2));
        assert_eq!(params.limit, Some(50));
    }
    
    #[test]
    fn test_get_collection_assets_params_defaults() {
        // Test with all None values
        let params = GetCollectionAssetsParams {
            search: None,
            sort_by: None,
            sort_order: None,
            page: None,
            limit: None,
        };
        
        assert_eq!(params.search, None);
        assert_eq!(params.sort_by, None);
        assert_eq!(params.sort_order, None);
        assert_eq!(params.page, None);
        assert_eq!(params.limit, None);
    }
}