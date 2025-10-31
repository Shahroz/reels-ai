//! Handler for deleting an item from a user DB collection.
//!
//! This endpoint allows an authenticated user to remove an item by its ID
//! from a collection they own.
//! Adheres to 'one item per file' and FQN guidelines.

/// Deletes an item from a user DB collection.
///
/// Permanently removes an item by its ID and parent collection ID,
/// ensuring the authenticated user owns the collection.
#[utoipa::path(
    delete,
    path = "/api/user-db-collections/{collection_id}/items/{item_id}",
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the parent collection"),
        ("item_id" = uuid::Uuid, Path, description = "ID of the item to delete")
    ),
    responses(
        (status = 204, description = "User DB collection item deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "User does not own the parent collection"),
        (status = 404, description = "Parent collection or item not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User DB Collection Items"
)]
#[actix_web::delete("/{item_id}")]
pub async fn delete_user_db_collection_item(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    path_params: actix_web::web::Path<(uuid::Uuid, uuid::Uuid)>,
) -> actix_web::HttpResponse {
    let user_id = claims.user_id;
    let (collection_id_uuid, item_id_uuid) = path_params.into_inner();

    match crate::queries::user_db_collections::items::delete_user_db_collection_item_query::delete_user_db_collection_item_query(
        pool.get_ref(),
        user_id,
        collection_id_uuid,
        item_id_uuid,
    ).await
    {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                actix_web::HttpResponse::NoContent().finish()
            } else {
                // This means item was not found in the specified collection, or it didn't belong to that collection.
                actix_web::HttpResponse::NotFound().json(
                    crate::routes::error_response::ErrorResponse {
                        error: std::string::String::from("Item not found in the specified collection or delete failed."),
                    },
                )
            }
        }
        Err(e) => {
            // Inspect the error to return a more specific HTTP status code
            let error_message = e.to_string();
            if error_message.contains("User does not own the parent collection.") {
                log::warn!("Forbidden attempt to delete item: {error_message}");
                actix_web::HttpResponse::Forbidden().json(
                    crate::routes::error_response::ErrorResponse {
                        error: std::string::String::from("User does not own the parent collection."),
                    },
                )
            } else if error_message.contains("Parent collection not found.") {
                log::warn!("Parent collection not found for item deletion: {error_message}");
                actix_web::HttpResponse::NotFound().json(
                    crate::routes::error_response::ErrorResponse {
                        error: std::string::String::from("Parent collection not found."),
                    },
                )
            } else {
                log::error!("Failed to delete user DB collection item via query: {e:?}");
                actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::error_response::ErrorResponse {
                        error: std::string::String::from("Failed to delete collection item."),
                    },
                )
            }
        }
    }
}
