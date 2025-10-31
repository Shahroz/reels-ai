//! Handler for listing items within a user DB collection.
//!
//! This endpoint retrieves all items belonging to a specific custom database
//! collection owned by the authenticated user. It supports pagination,
//! sorting, and searching of items.
//! Adheres to 'one item per file' and FQN guidelines.

//! Revision History
//! - 2025-05-06T21:08:30Z @AI: Implement pagination, sorting, and searching.

/// Parameters for listing user DB collection items with pagination, sorting, and search.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ListUserDbCollectionItemsParams {
    /// Page number for pagination.
    #[schema(example = 1, value_type = Option<i64>)]
    pub page: Option<i64>,
    /// Number of items per page.
    #[schema(example = 10, value_type = Option<i64>)]
    pub limit: Option<i64>,
    /// Field to sort by (e.g., "created_at", "updated_at"). Defaults to "created_at".
    #[schema(example = "created_at", value_type = Option<String>)]
    pub sort_by: Option<String>,
    /// Sort order ("asc" or "desc"). Defaults to "desc".
    #[schema(example = "desc", value_type = Option<String>)]
    pub sort_order: Option<String>,
    /// Search term to filter items by their content (item_data cast to text).
    #[schema(example = "keyword", value_type = Option<String>)]
    pub search: Option<String>,
}

/// Response for listing user DB collection items, including items and total count.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ListUserDbCollectionItemsResponse {
    /// The list of user DB collection items for the current page.
    pub items: std::vec::Vec<crate::db::user_db_collection_item::UserDbCollectionItem>,
    /// The total count of items matching the criteria.
    pub total_count: i64,
}

/// Lists all items in a user DB collection.
///
/// Retrieves all items for a given collection ID, provided the
/// authenticated user owns the collection. Supports pagination, sorting, and searching.
#[utoipa::path(
    get,
    path = "/api/user-db-collections/{collection_id}/items",
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the parent collection"),
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default 10)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (e.g., 'created_at', 'updated_at'; default 'created_at')"),
        ("sort_order" = Option<String>, Query, description = "Sort direction ('asc' or 'desc'; default 'desc')"),
        ("search" = Option<String>, Query, description = "Search term to filter items by content (searches item_data as text)")
    ),
    responses(
        (status = 200, description = "Paginated list of user DB collection items", body = ListUserDbCollectionItemsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "User does not own the parent collection"),
        (status = 404, description = "Parent collection not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User DB Collection Items"
)]
#[actix_web::get("")]
pub async fn list_user_db_collection_items(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    collection_id: actix_web::web::Path<uuid::Uuid>,
    params: actix_web::web::Query<ListUserDbCollectionItemsParams>,
) -> actix_web::HttpResponse {
    let user_id = claims.user_id;
    let collection_id_uuid = collection_id.into_inner();

   let page = params.page.unwrap_or(1).max(1);
   let limit = params.limit.unwrap_or(10).max(1);

    let sort_by_column_name = params.sort_by.as_deref().unwrap_or("created_at");

    let sort_order_param = params.sort_order.clone().unwrap_or_else(|| "desc".into());
    let sort_order = if sort_order_param.to_lowercase() == "asc" { "ASC" } else { "DESC" };
    
    // Prepare the search pattern. Only apply search if the string is not empty.
    // The query function expects a pattern for ILIKE.
    let search_pattern = params
        .search
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    match crate::queries::user_db_collections::items::list_user_db_collection_items_query::list_user_db_collection_items_query(
        pool.get_ref(),
        user_id,
        collection_id_uuid,
        page,
        limit,
        sort_by_column_name,
        sort_order,
        search_pattern.as_deref(),
    ).await {
        Ok((items, total_count)) => {
            actix_web::HttpResponse::Ok().json(ListUserDbCollectionItemsResponse { items, total_count })
        }
        Err(e) => {
            // Log the full error from the query function
            log::error!("Error in list_user_db_collection_items_query for collection {collection_id_uuid}: {e:?}");
            
            // Map specific error messages from query to HTTP responses
            let error_message = e.to_string();
            if error_message.starts_with("Forbidden:") {
                actix_web::HttpResponse::Forbidden().json(
                    crate::routes::error_response::ErrorResponse {
                        error: error_message, // Or a generic "Forbidden" message
                    })
            } else if error_message.starts_with("NotFound:") {
                actix_web::HttpResponse::NotFound().json(
                    crate::routes::error_response::ErrorResponse {
                        error: error_message, // Or a generic "Not Found" message
                    })
            } else {
                // For other errors, return InternalServerError
                actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::error_response::ErrorResponse {
                        error: std::string::String::from("An internal error occurred while listing collection items."),
                    })
            }
        }
    }
}
