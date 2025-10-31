//! Handles the HTTP GET request for retrieving a specific bundle by its ID.
//!
//! This module defines the Actix-web handler responsible for fetching a `Bundle`
//! item from the database, identified by its UUID. It ensures that the requesting
//! user owns the bundle before returning it. Adheres to project coding standards.

// Revision History
// - 2025-05-29T16:08:13Z @AI: Initial implementation of get_bundle_by_id handler.

// No `use` statements as per guidelines. Fully qualified paths will be used.

#[utoipa::path(
    get,
    path = "/api/bundles/{bundle_id}",
    params(
        ("bundle_id" = uuid::Uuid, Path, description = "Unique identifier of the bundle to retrieve")
    ),
    responses(
        (status = 200, description = "Bundle retrieved successfully", body = crate::db::bundles::Bundle),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User does not own this bundle or bundle is otherwise inaccessible", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "Bundle not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal Server Error", body = crate::routes::error_response::ErrorResponse)
    ),
    tag = "Bundles",
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::get("/{bundle_id}")]
pub async fn get_bundle_by_id(
    pool: actix_web::web::Data<sqlx::PgPool>,
    bundle_id_path: actix_web::web::Path<uuid::Uuid>,
   user: actix_web::web::ReqData<crate::middleware::auth::AuthenticatedUser>,
) -> impl actix_web::Responder {
    let bundle_id = bundle_id_path.into_inner();

    let authenticated_user = user.into_inner();
    let request_user_id = match authenticated_user {
        crate::middleware::auth::AuthenticatedUser::Jwt(claims) => claims.user_id,
       crate::middleware::auth::AuthenticatedUser::ApiKey(id) => id,
   };

   match crate::queries::bundles::find_bundle_by_id::find_bundle_by_id(pool.get_ref(), bundle_id).await {
       Ok(Some(bundle)) => {
           if bundle.user_id == request_user_id {
               actix_web::HttpResponse::Ok().json(bundle)
            } else {
                log::warn!(
                    "User {} attempted to access bundle {} owned by user {}. Access forbidden.",
                    request_user_id,
                    bundle_id,
                    bundle.user_id
                );
                actix_web::HttpResponse::Forbidden().json(crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("You do not have permission to access this bundle."),
                })
            }
        }
        Ok(None) => {
            log::info!("Bundle with ID {bundle_id} not found for user {request_user_id}.");
            actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Bundle not found."),
            })
        }
        Err(sqlx_error) => {
            log::error!(
                "Database error while user {request_user_id} attempting to find bundle {bundle_id}: {sqlx_error:?}"
            );
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("An internal server error occurred while retrieving the bundle."),
            })
        }
    }
}
