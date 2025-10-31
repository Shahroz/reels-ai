//! Handler for deleting a user DB collection.
//!
//! This endpoint allows an authenticated user to delete one of their custom
//! database collections by its ID.
//! Adheres to 'one item per file' and FQN guidelines.

/// Deletes a specific DB collection by ID.
///
/// Permanently removes a custom database collection if it exists and
/// belongs to the authenticated user.
#[utoipa::path(
    delete,
    path = "/api/user-db-collections/{collection_id}",
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the collection to delete")
    ),
    responses(
        (status = 204, description = "User DB collection deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Collection not found or not owned by user", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User DB Collections"
)]
#[actix_web::delete("/{collection_id}")]
pub async fn delete_user_db_collection(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    collection_id: actix_web::web::Path<uuid::Uuid>,
) -> actix_web::HttpResponse {
    let user_id = claims.user_id;
    let id_to_delete = collection_id.into_inner();

    match crate::queries::user_db_collections::delete_user_db_collection_query::delete_user_db_collection_query(
        pool.get_ref(),
        user_id,
        id_to_delete,
    )
    .await
    {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                actix_web::HttpResponse::NoContent().finish()
            } else {
                actix_web::HttpResponse::NotFound().json(
                    crate::routes::error_response::ErrorResponse {
                        error: std::string::String::from("Collection not found or not owned by user."),
                    },
                )
            }
        }
        Err(e) => {
            log::error!("Failed to delete user DB collection: {e:?}");
            // The new query function returns sqlx::Error directly.
            // Specific error handling (e.g., for not found vs. other DB errors) can be done here if needed.
            actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to delete collection."),
                },
            )
        }
    }
}
