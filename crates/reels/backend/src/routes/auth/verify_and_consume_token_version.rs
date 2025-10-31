//! Verifies and consumes magic link token version.
//!
//! Checks that the token version in the JWT matches the user's current version,
//! then atomically increments it to prevent token reuse.

/// Verifies token version and atomically increments it.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `user` - The user attempting to authenticate
/// * `claims_token_version` - Token version from JWT claims
///
/// # Returns
///
/// `Ok(())` if version matches and was successfully incremented
/// `Err(response)` if version mismatch or increment failed
///
/// # Security
///
/// - Uses compare-and-swap to prevent race conditions
/// - Ensures single-use magic links
/// - Atomic operation prevents concurrent token use
pub async fn verify_and_consume_token_version(
    pool: &sqlx::PgPool,
    user: &crate::db::users::User,
    claims_token_version: i32,
) -> std::result::Result<(), actix_web::HttpResponse> {
    // Verify token_version (single-use enforcement)
    if user.token_version != claims_token_version {
        log::warn!(
            "Token version mismatch for user {}: expected {}, got {} (token already used)",
            user.id,
            user.token_version,
            claims_token_version
        );
        return std::result::Result::Err(
            actix_web::HttpResponse::Unauthorized()
                .body("This login link has already been used or is no longer valid"),
        );
    }

    // Increment token_version (invalidate this magic link) with compare-and-swap
    log::debug!(
        "Incrementing token version for user: {} (expected: {})",
        user.id,
        user.token_version
    );

    if let std::result::Result::Err(e) =
        crate::queries::users::increment_token_version(pool, user.id, user.token_version).await
    {
        log::error!(
            "CRITICAL: Failed to increment token version for user {}: {} (token likely already used)",
            user.id,
            e
        );
        return std::result::Result::Err(
            actix_web::HttpResponse::Unauthorized().body("This login link has already been used"),
        );
    }

    log::info!(
        "Token version incremented for user: {} (now: {})",
        user.id,
        user.token_version + 1
    );

    std::result::Result::Ok(())
}

