//! API endpoint for verifying magic link tokens.
//!
//! This endpoint accepts a magic link JWT token, validates it, increments the user's
//! token_version to enforce one-time use, generates a session JWT, and returns both
//! the session token and user data as JSON.

/// Verify magic link token and return session token + user data
///
/// This endpoint validates a magic link JWT token and returns a session token
/// along with the user's profile data. It enforces one-time use by incrementing
/// the user's token_version.
///
/// # Security Features
///
/// - JWT signature and expiration validation
/// - Token type verification (must be "magic-link")
/// - One-time use enforcement via token_version
/// - Deactivated user check
/// - Login analytics tracking
#[utoipa::path(
    post,
    path = "/auth/verify-magic-link-token",
    tag = "Auth",
    request_body = crate::routes::auth::verify_magic_link_token_request::VerifyMagicLinkTokenRequest,
    responses(
        (status = 200, description = "Token verified, session created", body = crate::routes::auth::verify_magic_link_token_response::VerifyMagicLinkTokenResponse),
        (status = 400, description = "Invalid request format"),
        (status = 401, description = "Invalid, expired, or already-used token"),
        (status = 500, description = "Internal server error")
    )
)]
#[actix_web::post("/verify-magic-link-token")]
#[tracing::instrument(
    name = "verify_magic_link_token",
    skip(pool, req_body, http_req, session_manager),
    fields(token_length = req_body.token.len())
)]
pub async fn verify_magic_link_token(
    pool: actix_web::web::Data<sqlx::PgPool>,
    req_body: actix_web::web::Json<crate::routes::auth::verify_magic_link_token_request::VerifyMagicLinkTokenRequest>,
    #[cfg(feature = "events")]
    http_req: actix_web::HttpRequest,
    #[cfg(feature = "events")]
    session_manager: actix_web::web::Data<std::sync::Arc<crate::services::session_manager::HybridSessionManager>>,
) -> actix_web::HttpResponse {
    #[cfg(feature = "events")]
    let processing_start = std::time::Instant::now();

    log::debug!("Verifying magic link token");

    // 1-4. Core verification: validate token, fetch user, enforce single-use, create session
    let result = match crate::routes::auth::verify_magic_link_core::verify_magic_link_core(
        &pool,
        &req_body.token,
    )
    .await
    {
        std::result::Result::Ok(r) => r,
        std::result::Result::Err(e) => {
            return crate::routes::auth::map_verification_error_to_response::map_verification_error_to_response(e);
        }
    };

    let user = result.user;
    let session_token = result.session_token;

    // 5. Track login analytics event
    #[cfg(feature = "events")]
    {
        let request_context = crate::routes::auth::extract_request_context::extract_request_context_for_magic_link(
            &http_req,
            &session_manager,
        ).await;
        
        crate::routes::auth::track_magic_link_login_analytics::track_magic_link_login_analytics(
            &pool,
            user.id,
            &request_context,
            processing_start,
        ).await;
    }

    log::info!("User {} authenticated successfully via magic link", user.id);

    // 6. Return session token and user data as JSON
    let response = crate::routes::auth::verify_magic_link_token_response::VerifyMagicLinkTokenResponse {
        session_token,
        user: crate::db::users::PublicUser::from(user),
    };

    actix_web::HttpResponse::Ok().json(response)
}

