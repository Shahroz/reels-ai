//! Handler for retrieving a specific item from a user DB collection.
//!
//! This endpoint allows an authenticated user to fetch a single item by its ID,
//! from a collection they own.
//! Adheres to 'one item per file' and FQN guidelines.

/// Retrieves a specific item from a user DB collection.
///
/// Fetches an item by its ID and its parent collection's ID,
/// ensuring the authenticated user owns the collection.
#[utoipa::path(
    get,
    path = "/api/user-db-collections/{collection_id}/items/{item_id}",
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the parent collection"),
        ("item_id" = uuid::Uuid, Path, description = "ID of the item to retrieve")
    ),
    responses(
        (status = 200, description = "User DB collection item details", body = crate::db::user_db_collection_item::UserDbCollectionItem),
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
#[actix_web::get("/{item_id}")]
pub async fn get_user_db_collection_item(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    path_params: actix_web::web::Path<(uuid::Uuid, uuid::Uuid)>,
) -> actix_web::HttpResponse {
    let user_id = claims.user_id;
    let (collection_id_uuid, item_id_uuid) = path_params.into_inner();

    // 1. Verify ownership of parent collection
    match sqlx::query!(
        r#"SELECT user_id FROM user_db_collections WHERE id = $1"#,
        collection_id_uuid
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(record)) => {
            if record.user_id != user_id {
                return actix_web::HttpResponse::Forbidden().json(
                    crate::routes::error_response::ErrorResponse {
                        error: std::string::String::from("User does not own the parent collection."),
                    },
                );
            }
        }
        Ok(None) => {
            return actix_web::HttpResponse::NotFound().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Parent collection not found."),
                },
            );
        }
        Err(e) => {
            log::error!("Failed to fetch parent collection for getting item: {e:?}");
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to verify collection ownership."),
                },
            );
        }
    };

    // 2. Fetch item
    match sqlx::query_as!(
        crate::db::user_db_collection_item::UserDbCollectionItem,
        r#"
        SELECT id, user_db_collection_id, item_data, created_at, updated_at
        FROM user_db_collection_items
        WHERE id = $1 AND user_db_collection_id = $2
        "#,
        item_id_uuid,
        collection_id_uuid
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(item)) => actix_web::HttpResponse::Ok().json(item),
        Ok(None) => actix_web::HttpResponse::NotFound().json(
            crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Item not found in the specified collection."),
            },
        ),
        Err(e) => {
            log::error!("Failed to get user DB collection item: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to retrieve collection item."),
                },
            )
        }
    }
}
