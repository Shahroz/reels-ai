//! Handler for updating an organization via admin endpoint.
//!
//! This endpoint allows administrators to update organization name and/or transfer
//! ownership. The handler validates the request and delegates to the service layer
//! which handles the complete business operation including transaction management
//! and audit logging.

#[utoipa::path(
    patch,
    path = "/api/admin/organizations/{organization_id}",
    tag = "Admin",
    params(
        ("organization_id" = uuid::Uuid, Path, description = "Organization ID to update")
    ),
    request_body = crate::routes::admin::organizations::admin_update_organization_request::AdminUpdateOrganizationRequest,
    responses(
        (status = 200, description = "Organization updated successfully", body = crate::db::organizations::Organization),
        (status = 400, description = "Bad request - invalid input", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized - user is not an admin", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "Organization not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::patch("/{organization_id}")]
#[tracing::instrument(skip(pool, auth_claims, payload))]
pub async fn admin_update_organization_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    organization_id: actix_web::web::Path<uuid::Uuid>,
    payload: actix_web::web::Json<crate::routes::admin::organizations::admin_update_organization_request::AdminUpdateOrganizationRequest>,
) -> impl actix_web::Responder {
    match crate::queries::admin::organizations::services::update_organization_service(
        pool.get_ref(),
        auth_claims.user_id,
        organization_id.into_inner(),
        payload.name.clone(),
        payload.owner_user_id,
    )
    .await
    {
        Ok(org) => actix_web::HttpResponse::Ok().json(org),
        Err(e) => {
            let error_msg = e.to_string();

            if error_msg.contains("not found") {
                log::warn!(
                    "Admin {} tried to update non-existent organization: {}",
                    auth_claims.user_id,
                    error_msg
                );
                actix_web::HttpResponse::NotFound().json(
                    crate::routes::error_response::ErrorResponse {
                        error: error_msg,
                    },
                )
            } else if error_msg.contains("cannot be empty")
                || error_msg.contains("must be provided")
                || error_msg.contains("does not exist")
                || error_msg.contains("already the current owner")
            {
                log::warn!(
                    "Admin {} provided invalid input for updating organization: {}",
                    auth_claims.user_id,
                    error_msg
                );
                actix_web::HttpResponse::BadRequest().json(
                    crate::routes::error_response::ErrorResponse {
                        error: error_msg,
                    },
                )
            } else {
                log::error!(
                    "Admin {} failed to update organization: {}",
                    auth_claims.user_id,
                    e
                );
                actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::error_response::ErrorResponse {
                        error: String::from("Failed to update organization."),
                    },
                )
            }
        }
    }
}
