//! Generates a session JWT token for an authenticated user.
//!
//! Creates a 30-day session token that the user can use for subsequent requests.
//! This is called after successful authentication (magic link or password).

/// Generates a 30-day session JWT for a user.
///
/// # Arguments
///
/// * `user` - The authenticated user
///
/// # Returns
///
/// Result containing the JWT token string or a JWT error
pub fn generate_session_token_for_user(
    user: &crate::db::users::User,
) -> std::result::Result<std::string::String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now() + chrono::Duration::hours(24 * 30);
    let expiration_ts = expiration.timestamp() as u64;
    
    let session_claims = crate::auth::tokens::Claims {
        user_id: user.id,
        is_admin: user.is_admin,
        email: user.email.clone(),
        email_verified: user.email_verified,
        exp: expiration_ts,
        ..std::default::Default::default()
    };

    crate::auth::tokens::create_jwt(&session_claims)
}

