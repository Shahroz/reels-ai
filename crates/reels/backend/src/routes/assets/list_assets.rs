//! Defines the `list_assets` HTTP route handler.
//!
//! This handler retrieves a paginated and sorted list of assets for a user,
//! with collection details included when available.
//! Adheres to the project's Rust coding standards.


/// Query parameters for listing assets
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ListAssetsParams {
    #[serde(default = "default_search")]
    #[schema(example = "%%")]
    pub search: std::string::String,
    #[serde(default = "default_sort_by")]
    #[schema(example = "assets.created_at")]
    pub sort_by: std::string::String,
    #[serde(default = "default_sort_order")]
    #[schema(example = "DESC")]
    pub sort_order: std::string::String,
    #[serde(default = "default_page")]
    #[schema(example = 1)]
    pub page: i64,
    #[serde(default = "default_limit")]
    #[schema(example = 20)]
    pub limit: i64,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub collection_id: std::option::Option<uuid::Uuid>,
    #[schema(example = false)]
    pub is_public: std::option::Option<bool>,
    #[schema(example = false)]
    pub logo_related: std::option::Option<bool>,
}

fn default_search() -> std::string::String {
    "%%".to_string()
}

fn default_sort_by() -> std::string::String {
    "assets.created_at".to_string()
}

fn default_sort_order() -> std::string::String {
    "DESC".to_string()
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    10
}

/// List assets with collection details
///
/// Retrieves a paginated and sorted list of assets for the authenticated user.
/// If an asset belongs to a collection, the collection details will be included.
#[utoipa::path(
    get,
    path = "/api/assets",
    tag = "Assets",
    params(
        ("search" = Option<std::string::String>, Query, description = "Search pattern for asset names"),
        ("sort_by" = Option<std::string::String>, Query, description = "Sort field (assets.id, assets.name, assets.type, assets.created_at, assets.updated_at)"),
        ("sort_order" = Option<std::string::String>, Query, description = "Sort order (ASC or DESC)"),
        ("page" = Option<i64>, Query, description = "Page number (1-based)"),
        ("limit" = Option<i64>, Query, description = "Number of items per page"),
        ("collection_id" = Option<uuid::Uuid>, Query, description = "Filter by collection ID"),
        ("is_public" = Option<bool>, Query, description = "Filter by public assets"),
        ("logo_related" = Option<bool>, Query, description = "Filter by logo relation (true: only assets linked to logos, false: only assets not linked to logos)")
    ),
    responses(
        (status = 200, description = "Assets retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("user_auth" = [])
    )
)]
#[actix_web::get("")]
#[tracing::instrument(skip(pool, claims))]
pub async fn list_assets(
    pool: actix_web::web::Data<sqlx::PgPool>,
    query: actix_web::web::Query<ListAssetsParams>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    let params = query.into_inner();

    // Validate and sanitize parameters
    let search_pattern = if params.search.is_empty() || params.search == "%%" {
        "%%".to_string()
    } else {
        format!("%{}%", params.search)
    };

    let sort_by = match params.sort_by.as_str() {
        "assets.id" | "assets.name" | "assets.type" | "assets.created_at" | "assets.updated_at" => {
            params.sort_by
        }
        // Also accept simple field names without table prefix  
        "id" => "assets.id".to_string(),
        "name" => "assets.name".to_string(),
        "type" => "assets.type".to_string(),
        "created_at" => "assets.created_at".to_string(),
        "updated_at" => "assets.updated_at".to_string(),
        _ => "assets.created_at".to_string(),
    };

    let sort_order = match params.sort_order.to_uppercase().as_str() {
        "ASC" | "DESC" => params.sort_order.to_uppercase(),
        _ => "DESC".to_string(),
    };

    let page = params.page.max(1);
    let limit = params.limit.max(1).min(100);
    let offset = (page - 1) * limit;
    let is_public_filter = params.is_public;
    let logo_related_filter = params.logo_related;

    // Get user's organization memberships
    let org_memberships = match crate::queries::organizations::find_active_memberships_for_user(&pool, user_id).await {
        Ok(memberships) => memberships,
        Err(e) => {
            log::error!("Error fetching organization memberships for user {user_id}: {e}");
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to retrieve user organization memberships".to_string(),
            });
        }
    };
    let org_ids: std::vec::Vec<uuid::Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();

    // Execute queries
    let assets_result = crate::queries::assets::list_assets_with_collection::list_assets_with_collection(
        &pool,
        user_id,
        &search_pattern,
        &sort_by,
        &sort_order,
        limit,
        offset,
        params.collection_id,
        is_public_filter,
        &org_ids,
        logo_related_filter,
    ).await;

    let total_result = crate::queries::assets::count_assets::count_assets(&pool, user_id, &search_pattern, params.collection_id, is_public_filter, &org_ids, logo_related_filter).await;

    match (assets_result, total_result) {
        (Ok(assets), Ok(Some(total))) => {
            let total_pages = (total + limit - 1) / limit;

            let response = crate::routes::assets::responses::ListAssetsWithCollectionResponse {
                items: assets,
                total_count: total,
                page,
                limit,
                total_pages,
            };

            actix_web::HttpResponse::Ok().json(response)
        }
        (Ok(assets), Ok(None)) => {
            // No assets found, return empty response
            let response = crate::routes::assets::responses::ListAssetsWithCollectionResponse {
                items: assets,
                total_count: 0,
                page,
                limit,
                total_pages: 0,
            };

            actix_web::HttpResponse::Ok().json(response)
        }
        (Err(e), _) => {
            log::error!("Failed to fetch assets: {e}");
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to fetch assets".to_string(),
            })
        }
        (_, Err(e)) => {
            log::error!("Failed to count assets: {e}");
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to count assets".to_string(),
            })
        }
    }
}
