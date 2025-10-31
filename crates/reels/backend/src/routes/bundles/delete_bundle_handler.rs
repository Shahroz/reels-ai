//! Handles the HTTP DELETE request for deleting a specific bundle.
//!
//! This module defines the Actix-web handler responsible for processing
//! requests to delete a `Bundle` item from the database. It ensures
//! that the authenticated user owns the bundle before deletion.

// Revision History (New File)
// - 2025-05-29T16:10:44Z @AI: Initial implementation of delete_bundle handler.

#[utoipa::path(
    delete,
    path = "/api/bundles/{bundle_id}",
    params(
        ("bundle_id" = uuid::Uuid, Path, description = "Unique identifier of the bundle to delete")
    ),
    responses(
        (status = 204, description = "Bundle deleted successfully"),
        (status = 401, description = "Unauthorized"), // Middleware typically handles actual response
        (status = 404, description = "Bundle not found or not owned by user", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal Server Error", body = crate::routes::error_response::ErrorResponse)
    ),
    tag = "Bundles",
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::delete("/{bundle_id}")] // Path is relative to the /bundles scope
pub async fn delete_bundle(
    pool: actix_web::web::Data<sqlx::PgPool>,
    bundle_id_path: actix_web::web::Path<uuid::Uuid>,
   user: actix_web::web::ReqData<crate::middleware::auth::AuthenticatedUser>,
) -> impl actix_web::Responder {
    let bundle_id = bundle_id_path.into_inner();
    let user_id = match user.into_inner() {
        crate::middleware::auth::AuthenticatedUser::Jwt(claims) => claims.user_id,
       crate::middleware::auth::AuthenticatedUser::ApiKey(id) => id,
   };

   match crate::queries::bundles::delete_bundle::delete_bundle(pool.get_ref(), bundle_id, user_id).await {
       Ok(rows_affected) => {
           if rows_affected > 0 {
               actix_web::HttpResponse::NoContent().finish() // 204 No Content
            } else {
                // Bundle not found, or the user does not own it.
                // In either case, from the user's perspective, the resource is not available for deletion by them.
                log::warn!(
                    "Attempt to delete bundle {bundle_id} by user {user_id} resulted in 0 rows affected (not found or not owned)."
                );
                actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Bundle not found or not owned by user."),
                })
            }
        }
        Err(sqlx_error) => {
            log::error!(
                "Failed to delete bundle {bundle_id} for user {user_id}: {sqlx_error:?}"
            );
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Failed to delete bundle due to an internal server error."),
            })
        }
    }
}
