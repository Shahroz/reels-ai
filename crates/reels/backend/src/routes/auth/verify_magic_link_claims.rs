//! Verifies and decodes magic link JWT claims.
//!
//! Validates the JWT token and ensures it's the correct type for magic link authentication.
//! Returns decoded claims on success or an error response on failure.

/// Verifies magic link JWT and returns decoded claims.
///
/// # Arguments
///
/// * `token` - The JWT token string from the magic link
///
/// # Returns
///
/// `Ok(claims)` if token is valid and is a magic-link type
/// `Err(response)` with appropriate HTTP error response
///
/// # Security
///
/// - Validates JWT signature and expiration
/// - Checks token type is "magic-link"
/// - Logs security events at appropriate levels
pub fn verify_magic_link_claims(
    token: &str,
) -> std::result::Result<
    crate::auth::tokens::magic_link_claims::MagicLinkClaims,
    actix_web::HttpResponse,
> {
    // Verify and decode JWT
    let claims = match crate::auth::tokens::verify_magic_link_jwt(token) {
        std::result::Result::Ok(c) => c,
        std::result::Result::Err(e) => {
            log::warn!("Invalid magic link token: {}", e);
            return std::result::Result::Err(
                actix_web::HttpResponse::BadRequest().body("Invalid or expired login link"),
            );
        }
    };

    log::debug!("Magic link token verified for user_id: {}", claims.user_id);

    // Verify token type
    if claims.token_type != "magic-link" {
        log::warn!(
            "Wrong token type for user {}: expected 'magic-link', got '{}'",
            claims.user_id,
            claims.token_type
        );
        return std::result::Result::Err(
            actix_web::HttpResponse::BadRequest().body("Invalid token type"),
        );
    }

    std::result::Result::Ok(claims)
}

