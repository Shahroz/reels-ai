//! Handler for listing predefined collections with pagination, sorting, and searching.
//!
//! Handles GET requests to `/api/predefined-collections`. Retrieves a paginated, sorted, and filtered
//! list of predefined collections, returning both items and total count.
//! Adheres to relevant guidelines, balancing FQN with route handler conventions.

use crate::routes::error_response::ErrorResponse;

/// Query parameters for listing predefined collections.
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct ListPredefinedCollectionsParams {
    #[schema(example = 1, value_type = Option<i64>)]
    pub page: Option<i64>,
    #[schema(example = 10, value_type = Option<i64>)]
    pub limit: Option<i64>,
    #[schema(example = "created_at", value_type = Option<String>)]
    pub sort_by: Option<String>,
    #[schema(example = "desc", value_type = Option<String>)]
    pub sort_order: Option<String>,
    #[schema(example = "Product", value_type = Option<String>)]
    pub search: Option<String>,
}

/// Response structure for listing predefined collections.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListPredefinedCollectionsResponse {
    pub items: std::vec::Vec<crate::db::predefined_collection::PredefinedCollection>,
    pub total_count: i64,
}

/// Lists predefined collections with pagination, sorting, and searching.
///
/// Retrieves a paginated, sorted, and filtered list of predefined collections.
/// This endpoint is typically used to browse available template collections.
#[utoipa::path(
    get,
    path = "/api/predefined-collections",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default 10)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (e.g., name, created_at, updated_at; default created_at)"),
        ("sort_order" = Option<String>, Query, description = "Sort direction (asc or desc, default desc)"),
        ("search" = Option<String>, Query, description = "Filter collections by name (case-insensitive search)")
    ),
    responses(
        (status = 200, description = "List of predefined collections", body = ListPredefinedCollectionsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Predefined Collections"
)]
#[actix_web::get("")]
pub async fn list_predefined_collections(
    pool: actix_web::web::Data<sqlx::PgPool>,
    params: actix_web::web::Query<ListPredefinedCollectionsParams>,
) -> impl actix_web::Responder {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).max(1);
    let offset = (page - 1) * limit;

    // Basic validation for sort_by to prevent trivial SQL injection, default to 'created_at'
    let sort_by_input = params.sort_by.clone().unwrap_or_else(|| "created_at".into());
    let sort_by = match sort_by_input.as_str() {
        "name" => "name",
        "updated_at" => "updated_at",
        "created_at" => "created_at",
        _ => "created_at", // Default to created_at if an unknown field is provided
    };

    // Basic validation for sort_order, default to 'desc'
    let sort_order_input = params.sort_order.clone().unwrap_or_else(|| "desc".into());
    let sort_order = if sort_order_input.to_lowercase() == "asc" { "ASC" } else { "DESC" };

    let search_pattern = params
        .search
        .clone()
        .map(|s| format!("%{}%", s.replace('%', r"\%").replace('_', r"\_"))) // Escape wildcards for ILIKE
        .unwrap_or_else(|| "%".into());

    // Total count query
    let total_count_result = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM predefined_collections WHERE name ILIKE $1",
        &search_pattern
    )
    .fetch_one(pool.get_ref())
    .await;

    let total_count = match total_count_result {
        Ok(Some(count)) => count,
        Ok(None) => 0,
        Err(e) => {
            log::error!("Failed to count predefined collections: {e:?}");
            return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to count collections.".into(),
            });
        }
    };

    // Items query
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT id, name, description, schema_definition, ui_component_definition, created_at, updated_at \
         FROM predefined_collections \
         WHERE name ILIKE ",
    );
    query_builder.push_bind(&search_pattern);
    query_builder.push(" ORDER BY ");
    query_builder.push(sort_by);
    query_builder.push(" ");
    query_builder.push(sort_order);
    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    let items_result = query_builder
        .build_query_as::<crate::db::predefined_collection::PredefinedCollection>()
        .fetch_all(pool.get_ref())
        .await;

    match items_result {
        Ok(items) => actix_web::HttpResponse::Ok().json(ListPredefinedCollectionsResponse {
            items,
            total_count,
        }),
        Err(e) => {
            log::error!("Failed to fetch predefined collections: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve collections.".into(),
            })
        }
    }
} 