//! Handler for changing a user's password.
//!
//! This endpoint allows an authenticated user to change their own password
//! by providing their current password and a new one.

use crate::auth::tokens::Claims;
use crate::routes::auth::change_password_request::ChangePasswordRequest;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    post,
    path = "/auth/change-password",
    tag = "Auth",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully"),
        (status = 400, description = "Bad request, e.g., incorrect current password or new password does not meet policy", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[actix_web::post("/change-password")]
#[instrument(skip(pool, payload, claims))]
pub async fn change_password(
    pool: web::Data<PgPool>,
    payload: web::Json<ChangePasswordRequest>,
    claims: Claims,
) -> impl Responder {
    // Note: The user_management::change_user_password function is a placeholder
    // and is expected to be implemented in a subsequent step.
    match crate::user_management::change_user_password(
        &pool,
        claims.user_id,
        &payload.current_password,
        &payload.new_password,
    )
    .await
    {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::warn!("Failed to change password for user {}: {}", claims.user_id, e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "message": format!("Failed to change password: {}", e),
            }))
        }
    }
}