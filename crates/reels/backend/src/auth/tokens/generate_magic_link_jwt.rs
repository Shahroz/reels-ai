//! Generates JWT tokens for magic link authentication.
//!
//! Creates short-lived (15 minute) JWT tokens containing user identity
//! and current token version for single-use magic link authentication.
//! Uses the JWT_SECRET from environment variables for signing.

/// Generates a magic link JWT for the given user.
///
/// # Arguments
///
/// * `user` - The user for whom to generate the magic link token
///
/// # Returns
///
/// A `Result` containing the JWT string on success, or an error message on failure.
///
/// # Token Properties
///
/// - Expiration: 15 minutes from creation
/// - Type: "magic-link"
/// - Includes: user_id, email, token_version
/// - Single-use: Invalidated after use via token_version increment
#[tracing::instrument(skip(user))]
pub fn generate_magic_link_jwt(
    user: &crate::db::users::User,
) -> std::result::Result<std::string::String, std::string::String> {
    let expiration = chrono::Utc::now() + chrono::Duration::minutes(15);
    let claims = crate::auth::tokens::magic_link_claims::MagicLinkClaims {
        user_id: user.id,
        email: user.email.clone(),
        token_version: user.token_version,
        token_type: "magic-link".to_string(),
        exp: expiration.timestamp(),
    };

    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| "JWT_SECRET not set".to_string())?;
    
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("Failed to create JWT: {}", e))?;

    std::result::Result::Ok(token)
}

// Note: This function requires integration tests with JWT_SECRET configured.
// Unit tests should be added once JWT_SECRET is injected via dependency injection.
// See: TODO fix5 - Inject JWT_SECRET via config instead of env::var

