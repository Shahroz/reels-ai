//! Verifies and decodes magic link JWT tokens.
//!
//! Validates signature, expiration, and decodes claims from magic link tokens.
//! Uses the JWT_SECRET from environment variables for verification.

/// Verifies and decodes a magic link JWT.
///
/// # Arguments
///
/// * `token` - The JWT token string to verify
///
/// # Returns
///
/// A `Result` containing the decoded `MagicLinkClaims` on success, or an error message on failure.
///
/// # Errors
///
/// Returns an error if:
/// - Token signature is invalid
/// - Token has expired
/// - Token format is malformed
/// - JWT_SECRET environment variable is not set
#[tracing::instrument(skip(token))]
pub fn verify_magic_link_jwt(
    token: &str,
) -> std::result::Result<crate::auth::tokens::magic_link_claims::MagicLinkClaims, std::string::String> {
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| "JWT_SECRET not set".to_string())?;
    
    let validation = jsonwebtoken::Validation::default();
    
    let token_data = jsonwebtoken::decode::<crate::auth::tokens::magic_link_claims::MagicLinkClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| format!("Invalid token: {}", e))?;

    std::result::Result::Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_magic_link_jwt_invalid_token() {
        // Test that invalid tokens are rejected
        let invalid_token = "invalid.jwt.token";
        let result = verify_magic_link_jwt(invalid_token);
        
        // Should return an error for invalid token
        assert!(result.is_err(), "Invalid token should be rejected");
    }

    #[test]
    fn test_verify_magic_link_jwt_empty_token() {
        // Test that empty tokens are rejected
        let empty_token = "";
        let result = verify_magic_link_jwt(empty_token);
        
        // Should return an error for empty token
        assert!(result.is_err(), "Empty token should be rejected");
    }
}

