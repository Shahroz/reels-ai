//! Handler for the /auth/verify-token endpoint.
//!
//! This endpoint relies on the JwtMiddleware to authenticate the request.
//! If authentication succeeds (JWT or API Key), it fetches and returns the user's public details.
//! If authentication fails, the middleware returns 401 Unauthorized before this handler is called.

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;

use crate::db; // Ensure db module is accessible
use crate::routes::auth::verify_token_response::VerifyTokenResponse;

/// Verify Token Endpoint
///
/// Checks the validity of the provided authentication token (JWT or API Key via Bearer header)
/// and returns the associated user's public information if the token is valid.
/// Authentication is handled by middleware.
#[utoipa::path(
    get,
    path = "/auth/verify-token",
    tag = "Auth",
    responses(
        (status = 200, description = "Token verified successfully", body = VerifyTokenResponse),
        (status = 401, description = "Authentication failed (invalid/missing token)"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::get("/verify-token")]
#[instrument(skip(req, pool))]
pub async fn verify_token(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> impl Responder {
    // Manually extract and verify the JWT from the Authorization header
    let token = match req.headers().get(actix_web::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer ")) {
        Some(t) => {
            log::info!("Received token for verification: {}...", &t[..t.len().min(50)]);
            t.to_string()
        }
        None => {
            log::warn!("Authorization header missing or not Bearer for /auth/verify-token");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Authorization header missing or invalid."
            }));
        }
    };

    let claims = match crate::auth::tokens::verify_jwt(&token) {
        Ok(claims) => {
            log::info!("Token verified successfully for user: {}", claims.user_id);
            claims
        }
        Err(e) => {
            log::warn!("JWT verification failed in /auth/verify-token: {e:?}");
            log::warn!("Token that failed verification: {}...", &token[..token.len().min(50)]);
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "status": "error",
                "message": "Invalid or expired token."
            }));
        }
    };

    let impersonated_user_id = claims.user_id;

    // Fetch user details from the database
    let impersonated_user = match db::users::find_user_by_id(pool.get_ref(), impersonated_user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::warn!("User {impersonated_user_id} authenticated by token but not found in DB.");
            return HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": "User associated with token not found."
            }));
        }
        Err(e) => {
            log::error!("Database error fetching user {impersonated_user_id} in verify_token: {e}");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Failed to retrieve user details."
            }));
        }
    };

    let is_impersonating = claims.is_impersonating.unwrap_or(false);
    let mut original_admin_user = None;

    if is_impersonating {
        if let Some(admin_id) = claims.admin_id {
            match db::users::find_user_by_id(pool.get_ref(), admin_id).await {
                Ok(Some(admin)) => original_admin_user = Some(admin.into()),
                _ => {
                    log::error!("Could not find original admin user with id {admin_id}");
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "status": "error",
                        "message": "Could not find original admin user."
                    }));
                }
            }
        }
    }

    let public_user: db::users::PublicUser = impersonated_user.into();
    let response = VerifyTokenResponse {
        user: public_user,
        is_impersonating,
        original_admin_user,
    };

    HttpResponse::Ok().json(response)
}
