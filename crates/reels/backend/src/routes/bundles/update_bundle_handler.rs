//! Handles the HTTP PUT request for updating an existing bundle.
//!
//! This module defines the Actix-web handler responsible for processing
//! requests to update a `Bundle` item in the database. It ensures that the requesting
//! user owns the bundle before applying changes from the request payload.
//! Adheres to project coding standards.

// Revision History
// - 2025-05-29T16:09:23Z @AI: Initial implementation of update_bundle handler.

#[utoipa::path(
    put,
    path = "/api/bundles/{bundle_id}",
    params(
        ("bundle_id" = uuid::Uuid, Path, description = "Unique identifier of the bundle to update")
    ),
    request_body = crate::routes::bundles::update_bundle_request::UpdateBundleRequest,
    responses(
        (status = 200, description = "Bundle updated successfully", body = crate::db::bundles::Bundle),
        (status = 400, description = "Invalid input / Validation error", body = crate::routes::error_response::ErrorResponse), // Assuming validation might be added
        (status = 401, description = "Unauthorized"), // Handled by middleware
        (status = 403, description = "Forbidden - User does not own this bundle", body = crate::routes::error_response::ErrorResponse), // Covered by 404 if db query uses user_id
        (status = 404, description = "Bundle not found or not owned by user", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal Server Error", body = crate::routes::error_response::ErrorResponse)
    ),
    tag = "Bundles",
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::put("/{bundle_id}")]
pub async fn update_bundle(
    pool: actix_web::web::Data<sqlx::PgPool>,
    bundle_id_path: actix_web::web::Path<uuid::Uuid>,
    payload: actix_web::web::Json<crate::routes::bundles::update_bundle_request::UpdateBundleRequest>,
   user: actix_web::web::ReqData<crate::middleware::auth::AuthenticatedUser>,
) -> impl actix_web::Responder {
    let bundle_id = bundle_id_path.into_inner();

    let request_user_id = match user.into_inner() {
        crate::middleware::auth::AuthenticatedUser::Jwt(claims) => claims.user_id,
        crate::middleware::auth::AuthenticatedUser::ApiKey(id) => id,
    };

    let request_data = payload.into_inner();

    // Input validation could be performed here if `validator` crate is used on UpdateBundleRequest
    // For example:
    // if let Err(validation_errors) = request_data.validate() {
    //     return actix_web::HttpResponse::BadRequest().json(
    //         crate::routes::error_response::ErrorResponse::from(validation_errors.to_string())
   //     );
   // }

   match crate::queries::bundles::update_bundle::update_bundle(
       pool.get_ref(),
       bundle_id,
       request_user_id, // user_id for ownership check in the DB query
        request_data.name,
        request_data.description,
        request_data.style_id,
        request_data.document_ids,
        request_data.asset_ids,
        request_data.format_ids,
    )
    .await
    {
        Ok(bundle) => actix_web::HttpResponse::Ok().json(bundle),
        Err(sqlx::Error::RowNotFound) => {
            log::warn!(
                "Bundle with ID {bundle_id} not found or not owned by user {request_user_id}."
            );
            actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Bundle not found or you do not have permission to update it."),
            })
        }
        Err(sqlx_error) => {
            // Check for unique constraint violations if applicable, e.g., if bundle name must be unique per user
            // if let Some(db_err) = sqlx_error.as_database_error() {
            //     if db_err.is_unique_violation() {
            //         return actix_web::HttpResponse::Conflict().json(crate::routes::error_response::ErrorResponse {
            //             error: std::string::String::from("Failed to update bundle due to a conflict (e.g., name already exists)."),
            //         });
            //     }
            // }
            log::error!(
                "Failed to update bundle {bundle_id} for user {request_user_id}: {sqlx_error:?}"
            );
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Failed to update bundle due to an internal server error."),
            })
        }
    }
}
