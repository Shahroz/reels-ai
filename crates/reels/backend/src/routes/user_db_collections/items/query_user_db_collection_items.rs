//! Defines the handler for querying items in a user DB collection.
//!
//! This module provides the `query_user_db_collection_items` Actix web handler.
//! It allows users to retrieve a paginated and filtered list of items from a specified collection
//! based on a simple query string. Adheres to 'one item per file' and FQN guidelines.

use crate::routes::user_db_collections::items::query_user_db_collection_items_response::QueryUserDbCollectionItemsResponse;

/// Queries items within a specific user DB collection based on a query string.
///
/// Allows for paginated retrieval of items matching a simple query language.
/// The query targets fields within the `item_data` JSONB column.
#[utoipa::path(
    post,
    path = "/api/user-db-collections/{collection_id}/items/query",
    request_body = crate::routes::user_db_collections::items::query_user_db_collection_items_request::QueryUserDbCollectionItemsRequest,
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the collection to query items from")
    ),
    responses( (status = 200, description = "Successfully retrieved items", body = QueryUserDbCollectionItemsResponse), (status = 400, description = "Invalid query string or pagination parameters", body = crate::routes::error_response::ErrorResponse), (status = 401, description = "Unauthorized"), (status = 403, description = "User does not own the parent collection", body = crate::routes::error_response::ErrorResponse), (status = 404, description = "Parent collection not found", body = crate::routes::error_response::ErrorResponse), (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse) ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User DB Collection Items"
)]
#[actix_web::post("/query")]
#[allow(clippy::too_many_lines)] // Placeholder, to be reviewed after parser integration
pub async fn query_user_db_collection_items(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    collection_id_path: actix_web::web::Path<uuid::Uuid>,
    req_body: actix_web::web::Json<crate::routes::user_db_collections::items::query_user_db_collection_items_request::QueryUserDbCollectionItemsRequest>,
) -> actix_web::HttpResponse {
    let collection_id = collection_id_path.into_inner();
    let page = req_body.page.unwrap_or(1).max(1);
    let limit = req_body.limit.unwrap_or(10).max(1);

    match crate::queries::user_db_collections::items::query_user_db_collection_items_query::query_user_db_collection_items_query(
        pool.get_ref(),
        claims.user_id,
        collection_id,
        &req_body.query,
        page, // Convert to i64 for query function
        limit, // Convert to i64 for query function
    )
    .await
    {
        Ok((items, total_count)) => {
            actix_web::HttpResponse::Ok().json(
                crate::routes::user_db_collections::items::query_user_db_collection_items_response::QueryUserDbCollectionItemsResponse {
                    items,
                    total_count, // total_count is already i64 from query function
                },
            )
        }
        Err(e) => {
            // Check if the error has a specific status code context
           if let Some(status_code) = e.downcast_ref::<actix_web::http::StatusCode>() {
                // Try to get the ErrorResponse from the anyhow error's source chain
               if let Some(err_resp) = e.downcast_ref::<crate::routes::error_response::ErrorResponse>() {
                    return actix_web::HttpResponse::build(*status_code).json(err_resp);
               }
                // Fallback if ErrorResponse is not the direct cause but status code is present
               return actix_web::HttpResponse::build(*status_code).json(
                    crate::routes::error_response::ErrorResponse {
                        error: format!("An error occurred: {}", e.root_cause()),
                    }
                );
            }

            // Default to InternalServerError for other types of anyhow errors
            log::error!("Unhandled error querying user DB collection items: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(
                 crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("An internal error occurred while querying collection items."),
                }
            )
        }
    }
}

// Basic tests can be added here after parser integration
#[cfg(test)]
mod tests {
    // Placeholder for tests. Full testing requires mocking DB and auth.
    #[test]
    fn placeholder_test_route_handler_structure() {
        // Tests for the route handler would focus on request parsing, auth delegation,
        // and correct response formation based on the query function's result.
        assert!(true);
    }
}
