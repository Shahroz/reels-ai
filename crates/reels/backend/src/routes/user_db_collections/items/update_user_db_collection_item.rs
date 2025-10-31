//! Handler for updating an existing item within a user DB collection.
//!
//! This endpoint allows an authenticated user to modify an item in one of their
//! custom database collections. The item's data must conform to the collection's schema.
//! Adheres to 'one item per file' and FQN guidelines.

use crate::routes::user_db_collections::items::user_db_collection_item_request::UserDbCollectionItemRequest;

/// Updates an existing item in a user DB collection.
///
/// Modifies an item's `item_data`. The new data must be valid
/// according to the collection's `schema_definition`.
/// Ensures the authenticated user owns the collection.
#[utoipa::path(
    put,
    path = "/api/user-db-collections/{collection_id}/items/{item_id}",
    request_body = UserDbCollectionItemRequest,
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the parent collection"),
        ("item_id" = uuid::Uuid, Path, description = "ID of the item to update")
    ),
    responses(
        (status = 200, description = "User DB collection item updated successfully", body = crate::db::user_db_collection_item::UserDbCollectionItem),
        (status = 400, description = "Invalid request payload or data does not match schema", body = crate::routes::error_response::ErrorResponse),
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
#[actix_web::put("/{item_id}")]
pub async fn update_user_db_collection_item(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    path_params: actix_web::web::Path<(uuid::Uuid, uuid::Uuid)>,
    req_body: actix_web::web::Json<crate::routes::user_db_collections::items::user_db_collection_item_request::UserDbCollectionItemRequest>,
) -> actix_web::HttpResponse {
    let user_id = claims.user_id;
    let (collection_id_uuid, item_id_uuid) = path_params.into_inner();

    match crate::queries::user_db_collections::items::update_user_db_collection_item_query::update_user_db_collection_item_query(
        pool.get_ref(),
        user_id,
        collection_id_uuid,
        item_id_uuid,
        req_body.item_data.clone(),
    )
    .await {
        Ok(item) => actix_web::HttpResponse::Ok().json(item),
        Err(e) => {
            log::error!("Failed to update user DB collection item via query: {e:?}");
            let error_message = e.to_string();
            if error_message.contains("UserDoesNotOwnCollection") {
                actix_web::HttpResponse::Forbidden().json(crate::routes::error_response::ErrorResponse { error: error_message })
            } else if error_message.contains("CollectionNotFound") {
                actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse { error: error_message })
            } else if error_message.contains("ItemNotFound") {
                actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse { error: error_message })
            } else if error_message.contains("ValidationFailed") {
                actix_web::HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse { error: error_message })
            } else if error_message.contains("InvalidSchemaInDb") {
                actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse { error: error_message })
            } else {
                actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to update collection item."),
                })
            }
        }
    }
}
