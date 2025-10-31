use crate::auth::tokens::{create_jwt, Claims};
use crate::db::users::find_user_by_id;
use crate::routes::auth::logout_response_body::LogoutResponseBody;
use crate::routes::auth::standard_logout_response::StandardLogoutResponse;
use crate::routes::auth::stop_impersonation_response::StopImpersonationResponse;
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

const JWT_EXPIRATION_HOURS: i64 = 24; // Standard JWT expiration

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "Auth",
    responses(
        (status = 200, description = "Logout successful. Body structure depends on whether impersonation was active.", body = LogoutResponseBody),
        (status = 500, description = "Internal server error (e.g., failed to fetch admin details or create token).", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/logout")]
#[tracing::instrument(skip(pool, auth_claims))]
pub async fn logout(pool: web::Data<PgPool>, auth_claims: Claims) -> impl Responder {
    // Check if this is a request to stop an impersonation session.
    if auth_claims.is_impersonating == Some(true) {
        if let Some(original_admin_id) = auth_claims.admin_id {
            // This is an impersonation session being stopped.
            // 1. Fetch the original admin's details.
            let admin_user = match find_user_by_id(&pool, original_admin_id).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    log::error!(
                        "Attempted to stop impersonation, but original admin user {original_admin_id} not found."
                    );
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Original administrator account not found.".to_string(),
                    });
                }
                Err(e) => {
                    log::error!(
                        "Database error while fetching original admin user {original_admin_id}: {e}"
                    );
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to retrieve original admin details.".to_string(),
                    });
                }
            };

            // 2. Generate a new JWT for the original admin.
            let now = chrono::Utc::now();
            let exp = (now + chrono::Duration::hours(JWT_EXPIRATION_HOURS)).timestamp() as u64;

            let new_admin_claims = Claims {
                user_id: admin_user.id,
                is_admin: admin_user.is_admin, // Should be true
                email: admin_user.email.clone(),
                email_verified: admin_user.email_verified,
                exp,
                admin_id: None,
                is_impersonating: Some(false), // Explicitly set to false
                feature_flags: Some(admin_user.feature_flags.clone()),
            };

            let token = match create_jwt(&new_admin_claims) {
                Ok(t) => t,
                Err(e) => {
                    log::error!("Failed to create new JWT for admin {}: {}", admin_user.id, e);
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to generate new session token.".to_string(),
                    });
                }
            };

            // 3. Return the new token and admin user details.
            let response = LogoutResponseBody::ImpersonationStopped(StopImpersonationResponse {
                token,
                user: admin_user.into(),
                message: "Impersonation stopped. Returned to admin session.".to_string(),
            });
            return HttpResponse::Ok().json(response);
        }
    }

    // Standard logout for non-impersonating users or inconsistent state.
    // The client is responsible for discarding the token.
    let response = LogoutResponseBody::Standard(StandardLogoutResponse {
        message: "Logout successful.".to_string(),
    });

    HttpResponse::Ok().json(response)
}
