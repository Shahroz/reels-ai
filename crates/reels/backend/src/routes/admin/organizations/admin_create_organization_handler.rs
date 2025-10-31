//! Handler for creating an organization via admin endpoint.
//!
//! This endpoint allows administrators to create organizations with a specified owner.
//! The handler validates the request and delegates to the service layer which handles
//! the complete business operation including transaction management and audit logging.
//! Uses typed errors from AdminServiceError for clean error handling.
//!
//! Revision History:
//! - 2025-10-10: Refactored to use typed errors instead of string matching.

#[utoipa::path(
    post,
    path = "/api/admin/organizations",
    tag = "Admin",
    request_body = crate::routes::admin::organizations::admin_create_organization_request::AdminCreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created successfully", body = crate::db::organizations::Organization),
        (status = 400, description = "Bad request - invalid input", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized - user is not an admin", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("")]
#[tracing::instrument(skip(pool, auth_claims, payload))]
pub async fn admin_create_organization_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    payload: actix_web::web::Json<crate::routes::admin::organizations::admin_create_organization_request::AdminCreateOrganizationRequest>,
) -> impl actix_web::Responder {
    match crate::queries::admin::organizations::services::create_organization_service(
        pool.get_ref(),
        auth_claims.user_id,
        &payload.name,
        payload.owner_user_id,
    )
    .await
    {
        Ok(org) => actix_web::HttpResponse::Created().json(org),
        Err(e) => {
            if e.is_client_error() {
                log::warn!(
                    "Admin {} provided invalid input for creating organization: {}",
                    auth_claims.user_id,
                    e
                );
                actix_web::HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse {
                    error: e.to_string(),
                })
            } else {
                log::error!(
                    "Admin {} failed to create organization '{}': {}",
                    auth_claims.user_id,
                    payload.name,
                    e
                );
                actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                    error: String::from("Failed to create organization."),
                })
            }
        }
    }
}
