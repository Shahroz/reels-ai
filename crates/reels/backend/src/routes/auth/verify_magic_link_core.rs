//! Core magic link verification logic shared by both GET and POST endpoints.
//!
//! This module contains the shared verification flow that both `verify_magic_link`
//! (GET, HTML response) and `verify_magic_link_token` (POST, JSON response) use.
//! It performs JWT validation, user fetching, token version enforcement, and
//! session token generation, returning the authenticated user and session token.

/// Result of core magic link verification.
///
/// Contains the authenticated user and their new session token.
pub struct VerificationResult {
    pub user: crate::db::users::User,
    pub session_token: std::string::String,
}

/// Error type for core verification failures.
///
/// These errors map to different HTTP responses depending on the endpoint type.
#[derive(Debug)]
pub enum VerificationError {
    /// JWT token is invalid, expired, or has wrong type
    InvalidToken,
    /// User not found or deactivated
    UserNotFound,
    /// Token version mismatch (token already used)
    TokenAlreadyUsed,
    /// Failed to create session token
    SessionCreationFailed,
}

/// Core magic link verification logic.
///
/// This function performs the complete verification flow:
/// 1. Verify JWT token and extract claims
/// 2. Fetch user from database and validate status
/// 3. Verify and consume token version (single-use enforcement)
/// 4. Generate session JWT token
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `token` - Magic link JWT token to verify
///
/// # Returns
///
/// * `Ok(VerificationResult)` - User authenticated, contains user data and session token
/// * `Err(VerificationError)` - Verification failed at some step
///
/// # Security
///
/// - Validates JWT signature and expiration
/// - Enforces single-use via token_version with race condition prevention
/// - Checks user status (rejects deactivated users)
/// - Increments token_version atomically using compare-and-swap
pub async fn verify_magic_link_core(
    pool: &sqlx::PgPool,
    token: &str,
) -> std::result::Result<VerificationResult, VerificationError> {
    // 1. Verify JWT token and claims
    let claims = crate::routes::auth::verify_magic_link_claims::verify_magic_link_claims(token)
        .map_err(|_| VerificationError::InvalidToken)?;

    // 2. Fetch and validate user
    let user = crate::routes::auth::fetch_user_for_magic_link::fetch_user_for_magic_link(
        pool,
        claims.user_id,
    )
    .await
    .map_err(|_| VerificationError::UserNotFound)?;

    // 3. Verify and consume token version (single-use enforcement)
    crate::routes::auth::verify_and_consume_token_version::verify_and_consume_token_version(
        pool,
        &user,
        claims.token_version,
    )
    .await
    .map_err(|_| VerificationError::TokenAlreadyUsed)?;

    // 4. Generate session JWT
    let session_token = crate::routes::auth::create_session_token::create_session_token(&user)
        .map_err(|_| VerificationError::SessionCreationFailed)?;

    std::result::Result::Ok(VerificationResult {
        user,
        session_token,
    })
}

