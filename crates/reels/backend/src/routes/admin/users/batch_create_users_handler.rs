//! Handler for batch creating users via admin endpoint.
//!
//! This endpoint allows administrators to create multiple users at once by providing
//! a list of email addresses. Returns 207 Multi-Status with detailed success/failure
//! results for each email, allowing partial success. The handler delegates to the service
//! layer which handles the complete business operation including transaction management
//! and comprehensive audit logging for compliance.
//!
//! Revision History:
//! - 2025-10-10: Initial creation with typed error handling and service layer pattern.

#[utoipa::path(
    post,
    path = "/api/admin/users/batch",
    tag = "Admin",
    request_body = crate::routes::admin::users::batch_create_users_request::BatchCreateUsersRequest,
    responses(
        (status = 207, description = "Multi-Status - partial success", body = crate::routes::admin::users::batch_create_users_response::BatchCreateUsersResponse),
        (status = 400, description = "Bad request - invalid input", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized - user is not an admin", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/batch")]
#[tracing::instrument(skip(pool, auth_claims, payload))]
pub async fn batch_create_users_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    payload: actix_web::web::Json<crate::routes::admin::users::batch_create_users_request::BatchCreateUsersRequest>,
) -> impl actix_web::Responder {
    match crate::queries::admin::users::services::batch_create_users_service(
        pool.get_ref(),
        auth_claims.user_id,
        payload.emails.clone(),
    )
    .await
    {
        Ok(result) => {
            let success_dtos: Vec<crate::routes::admin::users::batch_create_users_response::UserCreateSuccess> = result
                .success
                .into_iter()
                .map(|s| crate::routes::admin::users::batch_create_users_response::UserCreateSuccess {
                    email: s.email,
                    user: s.user,
                })
                .collect();

            let failed_dtos: Vec<crate::routes::admin::users::batch_create_users_response::UserCreateFailure> = result
                .failed
                .into_iter()
                .map(|f| crate::routes::admin::users::batch_create_users_response::UserCreateFailure {
                    email: f.email,
                    reason: f.reason,
                })
                .collect();

            let response = crate::routes::admin::users::batch_create_users_response::BatchCreateUsersResponse {
                success: success_dtos,
                failed: failed_dtos,
            };

            actix_web::HttpResponse::MultiStatus().json(response)
        }
        Err(e) => {
            let error_msg = e.to_string();

            if error_msg.contains("must be provided") {
                log::warn!(
                    "Admin {} provided invalid input for batch creating users: {}",
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
                    "Admin {} failed to batch create users: {}",
                    auth_claims.user_id,
                    e
                );
                actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::error_response::ErrorResponse {
                        error: String::from("Failed to create users."),
                    },
                )
            }
        }
    }
}

