//! Handler for creating a new item within a user DB collection.
//!
//! This endpoint allows an authenticated user to add an item to one of their
//! custom database collections. The item's data must conform to the collection's schema.
//! Adheres to 'one item per file' and FQN guidelines.

use crate::routes::user_db_collections::items::user_db_collection_item_request::UserDbCollectionItemRequest;

/// Creates a new item in a user DB collection.
///
/// Authenticated users can add items to their collections. The `item_data`
/// must be valid according to the collection's `schema_definition`.
#[utoipa::path(
    post,
    path = "/api/user-db-collections/{collection_id}/items",
    request_body = UserDbCollectionItemRequest,
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the parent collection")
    ),
    responses(
        (status = 201, description = "User DB collection item created successfully", body = crate::db::user_db_collection_item::UserDbCollectionItem),
        (status = 400, description = "Invalid request payload or data does not match schema", body = crate::routes::error_response::ErrorResponse),
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
#[actix_web::post("")]
pub async fn create_user_db_collection_item(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    collection_id: actix_web::web::Path<uuid::Uuid>,
    req_body: actix_web::web::Json<crate::routes::user_db_collections::items::user_db_collection_item_request::UserDbCollectionItemRequest>,
) -> actix_web::HttpResponse {
    let user_id = claims.user_id;
    let collection_id_uuid = collection_id.into_inner();

    match crate::queries::user_db_collections::items::create_user_db_collection_item_query::create_user_db_collection_item_query(
        pool.get_ref(),
        user_id,
        collection_id_uuid,
        req_body.item_data.clone(),
    )
    .await {
        Ok(item) => actix_web::HttpResponse::Created().json(item),
        Err(e) => {
            log::error!("Failed to create user DB collection item via query: {e:?}");
            if let Some(query_error) = e.downcast_ref::<crate::queries::user_db_collections::items::create_user_db_collection_item_query::QueryError>() {
                match query_error {
                    crate::queries::user_db_collections::items::create_user_db_collection_item_query::QueryError::Forbidden(msg) => {
                        actix_web::HttpResponse::Forbidden().json(crate::routes::error_response::ErrorResponse { error: msg.clone() })
                    }
                    crate::queries::user_db_collections::items::create_user_db_collection_item_query::QueryError::NotFound(msg) => {
                        actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse { error: msg.clone() })
                    }
                    crate::queries::user_db_collections::items::create_user_db_collection_item_query::QueryError::Validation(msg) => {
                        actix_web::HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse { error: msg.clone() })
                    }
                    _ => actix_web::HttpResponse::InternalServerError().json(
                        crate::routes::error_response::ErrorResponse {
                            error: std::string::String::from("Failed to create collection item due to an internal error."),
                        },
                    ),
                }
            } else {
                actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::error_response::ErrorResponse {
                        error: std::string::String::from("An unexpected error occurred."),
                    },
                )
            }
        }
    }
}
