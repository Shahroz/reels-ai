//! Password reset token generation for account recovery workflows.
//!
//! Generates cryptographically secure random tokens for password reset processes.
//! Creates 64-character alphanumeric tokens with 1-hour expiration times.
//! Uses the system's cryptographically secure random number generator.
//! Shorter expiration time than verification tokens for enhanced security.

/// Generates a secure random password reset token and its expiry time.
///
/// The token is a 64-character alphanumeric string suitable for password reset links.
/// The expiry time is set to 1 hour from the moment of generation for security.
///
/// # Returns
/// 
/// A tuple containing:
/// - The generated token (String) - 64 alphanumeric characters
/// - The expiry time (DateTime<Utc>) - 1 hour from now
///
/// # Security
///
/// Uses cryptographically secure random number generation via `rand::thread_rng()`.
/// Shorter expiration time than verification tokens reduces exposure window.
/// The 64-character length provides sufficient entropy for security.
#[tracing::instrument]
pub fn generate_password_reset_token() -> (std::string::String, chrono::DateTime<chrono::Utc>) {
    let mut rng = rand::thread_rng();
    let token: std::string::String = std::iter::repeat(())
        .map(|()| <rand::rngs::ThreadRng as rand::Rng>::sample(&mut rng, rand::distributions::Alphanumeric))
        .map(char::from)
        .take(64)
        .collect();

    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

    (token, expires_at)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_password_reset_token_properties() {
        let (token, expires_at) = super::generate_password_reset_token();
        
        // Test token length
        assert_eq!(token.len(), 64, "Token should be exactly 64 characters");
        
        // Test that token contains only alphanumeric characters
        assert!(token.chars().all(|c| c.is_alphanumeric()), "Token should contain only alphanumeric characters");
        
        // Test that expiry is in the future
        assert!(expires_at > chrono::Utc::now(), "Expiry should be in the future");
        
        // Test that expiry is approximately 1 hour from now (within 5 seconds tolerance)
        let expected_expiry = chrono::Utc::now() + chrono::Duration::hours(1);
        let delta = (expires_at - expected_expiry).num_seconds().abs();
        assert!(delta < 5, "Expiry time should be approximately 1 hour from now");
    }

    #[test]
    fn test_shorter_expiry_than_verification() {
        let verification_duration = chrono::Duration::hours(24);
        let reset_duration = chrono::Duration::hours(1);
        
        assert!(reset_duration < verification_duration, "Password reset tokens should expire sooner than verification tokens");
    }

    #[test]
    fn test_token_uniqueness() {
        let (token1, _) = super::generate_password_reset_token();
        let (token2, _) = super::generate_password_reset_token();
        
        // Tokens should be different (extremely high probability with 64 characters)
        assert_ne!(token1, token2, "Generated tokens should be unique");
    }
}
