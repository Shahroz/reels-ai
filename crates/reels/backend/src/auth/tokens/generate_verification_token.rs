//! Verification token generation for email verification workflows.
//!
//! Generates cryptographically secure random tokens for email verification processes.
//! Creates 64-character alphanumeric tokens with 24-hour expiration times.
//! Uses the system's cryptographically secure random number generator.
//! Tokens are designed to be URL-safe and easy to handle in email links.

/// Generates a secure random verification token and its expiry time.
///
/// The token is a 64-character alphanumeric string suitable for email verification.
/// The expiry time is set to 24 hours from the moment of generation.
///
/// # Returns
/// 
/// A tuple containing:
/// - The generated token (String) - 64 alphanumeric characters
/// - The expiry time (DateTime<Utc>) - 24 hours from now
///
/// # Security
///
/// Uses cryptographically secure random number generation via `rand::thread_rng()`.
/// The 64-character length provides sufficient entropy for security.
#[tracing::instrument]
pub fn generate_verification_token() -> (std::string::String, chrono::DateTime<chrono::Utc>) {
    let mut rng = rand::thread_rng();
    let token: std::string::String = std::iter::repeat(())
        .map(|()| <rand::rngs::ThreadRng as rand::Rng>::sample(&mut rng, rand::distributions::Alphanumeric))
        .map(char::from)
        .take(64)
        .collect();

    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

    (token, expires_at)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_verification_token_properties() {
        let (token, expires_at) = super::generate_verification_token();
        
        // Test token length
        assert_eq!(token.len(), 64, "Token should be exactly 64 characters");
        
        // Test that token contains only alphanumeric characters
        assert!(token.chars().all(|c| c.is_alphanumeric()), "Token should contain only alphanumeric characters");
        
        // Test that expiry is in the future
        assert!(expires_at > chrono::Utc::now(), "Expiry should be in the future");
        
        // Test that expiry is approximately 24 hours from now (within 5 seconds tolerance)
        let expected_expiry = chrono::Utc::now() + chrono::Duration::hours(24);
        let delta = (expires_at - expected_expiry).num_seconds().abs();
        assert!(delta < 5, "Expiry time should be approximately 24 hours from now");
    }

    #[test]
    fn test_token_uniqueness() {
        let (token1, _) = super::generate_verification_token();
        let (token2, _) = super::generate_verification_token();
        
        // Tokens should be different (extremely high probability with 64 characters)
        assert_ne!(token1, token2, "Generated tokens should be unique");
    }
}
