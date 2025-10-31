///! Defines handler for listing User DB Collections with pagination, sorting, and searching.
///!
///! Handles GET requests to `/api/user-db-collections`. Retrieves a paginated, sorted, and filtered
///! list of User DB Collections for the authenticated user, returning both items and total count.
///! Adheres to relevant guidelines, balancing FQN with route handler conventions.

use crate::routes::error_response::ErrorResponse;

/// Represents a UserDbCollection enriched with the count of items within it.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, utoipa::ToSchema)]
pub struct UserDbCollectionWithItemCount {
    #[schema(format = "uuid", value_type=String)]
    pub id: uuid::Uuid,
    #[schema(format = "uuid", value_type=String)]
    pub user_id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub schema_definition: serde_json::Value,
    #[schema(format = "uuid", value_type=String, nullable = true)]
    pub source_predefined_collection_id: Option<uuid::Uuid>,
    pub ui_component_definition: serde_json::Value,
    #[schema(format = "date-time", value_type=String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(format = "date-time", value_type=String)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[schema(example = 10)]
    pub items_count: i64,
}

/// Query parameters for listing user DB collections.
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct ListUserDbCollectionsParams {
    #[schema(example = 1, value_type = Option<i64>)]
    pub page: Option<i64>,
    #[schema(example = 10, value_type = Option<i64>)]
    pub limit: Option<i64>,
    #[schema(example = "created_at", value_type = Option<String>)]
    pub sort_by: Option<String>,
    #[schema(example = "desc", value_type = Option<String>)]
    pub sort_order: Option<String>,
    #[schema(example = "MyCollectionName", value_type = Option<String>)]
    pub search: Option<String>,
}

/// Response structure for listing user DB collections.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListUserDbCollectionsResponse {
    pub items: std::vec::Vec<UserDbCollectionWithItemCount>,
    pub total_count: i64,
}

/// Lists User DB Collections with pagination, sorting, and searching.
///
/// Retrieves a paginated, sorted, and filtered list of custom database collections
/// for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/user-db-collections",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default 10)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (e.g., name, created_at, updated_at; default created_at)"),
        ("sort_order" = Option<String>, Query, description = "Sort direction (asc or desc, default desc)"),
        ("search" = Option<String>, Query, description = "Filter collections by name (case-insensitive search)")
    ),
    responses(
        (status = 200, description = "List of user DB collections with item counts", body = ListUserDbCollectionsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User DB Collections"
)]
#[actix_web::get("")]
pub async fn list_user_db_collections(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    params: actix_web::web::Query<ListUserDbCollectionsParams>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
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
        "SELECT COUNT(*) FROM user_db_collections WHERE user_id = $1 AND name ILIKE $2",
        user_id,
        &search_pattern
    )
    .fetch_one(pool.get_ref())
    .await;

    let total_count = match total_count_result {
        Ok(Some(count)) => count,
        Ok(None) => 0,
        Err(e) => {
            log::error!(
                "Failed to count user DB collections for user {user_id}: {e:?}"
            );
            return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to count collections.".into(),
            });
        }
    };

    // Items query
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT c.id, c.user_id, c.name, c.description, c.schema_definition, c.source_predefined_collection_id, c.ui_component_definition, c.created_at, c.updated_at, \
         COUNT(i.id)::bigint as items_count \
         FROM user_db_collections c \
         LEFT JOIN user_db_collection_items i ON c.id = i.user_db_collection_id \
         WHERE c.user_id = ",
    );
    query_builder.push_bind(user_id);
    query_builder.push(" AND c.name ILIKE ");
    query_builder.push_bind(&search_pattern);
    query_builder.push(" GROUP BY c.id ORDER BY c.");
    query_builder.push(sort_by);
    query_builder.push(" ");
    query_builder.push(sort_order);
    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    let items_result = query_builder
        .build_query_as::<UserDbCollectionWithItemCount>()
        .fetch_all(pool.get_ref())
        .await;

    match items_result {
        Ok(items) => actix_web::HttpResponse::Ok().json(ListUserDbCollectionsResponse {
            items,
            total_count,
        }),
        Err(e) => {
            log::error!(
                "Failed to fetch user DB collections for user {user_id}: {e:?}"
            );
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve collections.".into(),
            })
        }
    }
}
