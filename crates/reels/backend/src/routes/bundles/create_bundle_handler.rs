//! Handles the HTTP POST request for creating a new bundle.
//!
//! This module defines the Actix-web handler responsible for processing
//! requests to create a `Bundle` item in the database. It extracts user
//! information from authentication middleware and uses data from the request
//! payload to populate and persist the new bundle.

// Revision History
// - 2025-05-29T16:03:50Z @AI: Initial implementation of create_bundle handler.

#[utoipa::path(
    post,
    path = "/api/bundles",
    request_body = crate::routes::bundles::create_bundle_request::CreateBundleRequest,
    responses(
        (status = 201, description = "Bundle created successfully", body = crate::db::bundles::Bundle),
        (status = 400, description = "Invalid input / Validation error", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized"), // Typically no body if middleware handles this
        (status = 409, description = "Conflict - Bundle with this name already exists", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal Server Error", body = crate::routes::error_response::ErrorResponse)
    ),
    tag = "Bundles",
    security(
        ("bearer_auth" = []) // Assumes "bearer_auth" is defined in ApiDoc's security_schemes
    )
)]
#[actix_web::post("")] // Path relative to the scope where this service is registered
pub async fn create_bundle(
    pool: actix_web::web::Data<sqlx::PgPool>,
    payload: actix_web::web::Json<crate::routes::bundles::create_bundle_request::CreateBundleRequest>,
   user: actix_web::web::ReqData<crate::middleware::auth::AuthenticatedUser>,
) -> impl actix_web::Responder {
    let user_id = match user.into_inner() {
        crate::middleware::auth::AuthenticatedUser::Jwt(claims) => claims.user_id,
        crate::middleware::auth::AuthenticatedUser::ApiKey(id) => id,
    };

    // Assuming validator::Validate derive on CreateBundleRequest is handled by Actix framework
    // or by explicit call:
    // if let Err(validation_errors) = payload.validate() {
    //     return actix_web::HttpResponse::BadRequest().json(
    //         crate::routes::error_response::ErrorResponse::from(validation_errors.to_string())
    //     );
    // }

    let request_data = payload.into_inner();

    let document_ids = request_data.document_ids.unwrap_or_default();
   let asset_ids = request_data.asset_ids.unwrap_or_default();
   let format_ids = request_data.format_ids.unwrap_or_default();

   match crate::queries::bundles::create_bundle::create_bundle(
       pool.get_ref(),
       user_id,
       &request_data.name,
        request_data.description.as_deref(),
        request_data.style_id,
        &document_ids,
        &asset_ids,
        &format_ids,
    )
    .await
    {
        Ok(bundle) => actix_web::HttpResponse::Created().json(bundle),
        Err(sqlx_error) => {
            if let Some(db_err) = sqlx_error.as_database_error() {
                if db_err.is_unique_violation() {
                    // This is a generic check for unique violation.
                    // Specific constraint name checks (e.g., for user_id + name uniqueness) might be needed if there are multiple unique constraints.
                    return actix_web::HttpResponse::Conflict().json(crate::routes::error_response::ErrorResponse {
                        error: std::string::String::from("A resource with conflicting unique attributes (e.g., name) already exists."),
                    });
                }
            }
            log::error!("Failed to create bundle: {sqlx_error:?}");
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Failed to create bundle due to an internal server error."),
            })
        }
    }
}
