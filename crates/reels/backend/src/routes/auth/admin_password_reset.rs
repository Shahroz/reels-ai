//! Handler for an admin to trigger a password reset for a user.
//!
//! This endpoint allows an administrator to initiate the password reset process
//! for any user, which would typically generate and send a reset link.

use crate::auth::tokens::{self, Claims};
use crate::db::{password_resets, users};
use crate::email_service;
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/admin/users/{user_id}/reset-password",
    tag = "Admin",
    params(
        ("user_id" = Uuid, Path, description = "The ID of the user to trigger a password reset for")
    ),
    responses(
        (status = 200, description = "Password reset process successfully initiated"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/{user_id}/reset-password")]
#[instrument(skip(pool, auth_claims))]
pub async fn admin_password_reset(
    pool: web::Data<PgPool>,
    postmark_client: web::Data<std::sync::Arc<postmark::reqwest::PostmarkClient>>,
    auth_claims: Claims,
    user_id: web::Path<Uuid>,
) -> impl Responder {
    if !auth_claims.is_admin {
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "User is not authorized to perform this action.".to_string(),
        });
    }

    let user_id_val = user_id.into_inner();

    // Fetch the user to get their email address
    let user = match users::find_user_by_id(&pool, user_id_val).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::warn!(
                "Admin user '{}' attempted to reset password for non-existent user '{}'.",
                auth_claims.user_id,
                user_id_val
            );
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "User not found.".to_string(),
            });
        }
        Err(e) => {
            log::error!("Failed to fetch user '{}': {}", user_id_val, e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to initiate password reset.".to_string(),
            });
        }
    };

    // Generate a password reset token (valid for 1 hour)
    let (token, expires_at) = tokens::generate_password_reset_token();

    // Store the token in the password_reset_tokens table
    match password_resets::store_reset_token(&pool, user_id_val, &token, expires_at).await {
        Ok(_) => {
            log::info!(
                "Admin user '{}' initiated password reset for user '{}' ({})",
                auth_claims.user_id,
                user_id_val,
                user.email
            );

            // Send the password reset email
            match email_service::send_password_reset_email(&postmark_client, user_id_val, &user.email, &token).await
            {
                Ok(_) => {
                    log::info!("Password reset email sent to {}", user.email);
                    HttpResponse::Ok().finish()
                }
                Err(e) => {
                    log::error!("Failed to send password reset email to {}: {}", user.email, e);
                    // Return success anyway since token was stored - admin can manually provide link if needed
                    HttpResponse::Ok().finish()
                }
            }
        }
        Err(e) => {
            log::error!("Failed to store password reset token for user '{}': {}", user_id_val, e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to initiate password reset.".to_string(),
            })
        }
    }
}