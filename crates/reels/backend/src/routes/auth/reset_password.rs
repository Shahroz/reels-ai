//! Handler for the password reset endpoint.
//!
//! This endpoint allows a user to reset their password using a token
//! they received via email.

use crate::routes::auth::reset_password_request::ResetPasswordRequest;
use crate::routes::error_response::ErrorResponse;
use crate::user_management;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully"),
        (status = 400, description = "Invalid or expired token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Auth"
)]
#[instrument(skip(pool, payload), name = "reset_password_handler")]
#[actix_web::post("/reset-password")]
pub async fn reset_password(
    pool: web::Data<PgPool>,
    payload: web::Json<ResetPasswordRequest>,
) -> impl Responder {
    match user_management::reset_password(&pool, &payload.token, &payload.password).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            let err_msg = e.to_string();
            // Return 400 for invalid/expired token or password validation failures
            if err_msg.contains("Invalid or expired") || err_msg.contains("Password must") {
                HttpResponse::BadRequest().json(ErrorResponse { error: err_msg })
            } else {
                tracing::error!("Failed to reset password: {}", e);
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "An internal error occurred while resetting the password.".to_string(),
                })
            }
        }
    }
}
