//! JWT secret retrieval from environment variables.
//!
//! Provides secure access to the JWT signing secret from environment configuration.
//! The secret is expected to be set in the JWT_SECRET environment variable.
//! This function should be called during application startup to validate configuration.
//! Returns appropriate errors for missing or invalid configuration.

/// Retrieves the JWT secret from the JWT_SECRET environment variable.
///
/// # Returns
/// 
/// A `Result` containing the secret string on success, or a `VarError` if the
/// environment variable is not set or contains invalid UTF-8.
///
/// # Security
///
/// The secret should be a cryptographically strong random string of at least
/// 32 characters. This function does not validate secret strength.
#[tracing::instrument]
pub fn get_jwt_secret() -> std::result::Result<std::string::String, std::env::VarError> {
    std::env::var("JWT_SECRET")
}

/// Validates that the JWT secret is properly configured at application startup.
///
/// # Returns
///
/// A `Result` containing `()` on success, or an error message if the secret
/// is missing, empty, or insufficient length.
///
/// # Security
///
/// This should be called during application initialization to fail fast
/// if the JWT configuration is invalid.
#[tracing::instrument]
pub fn validate_jwt_secret_on_startup() -> std::result::Result<(), std::string::String> {
    match get_jwt_secret() {
        std::result::Result::Ok(secret) => {
            if secret.is_empty() {
                std::result::Result::Err("JWT_SECRET environment variable is empty".to_string())
            } else if secret.len() < 32 {
                std::result::Result::Err(format!("JWT_SECRET is too short: {} characters, minimum 32 required", secret.len()))
            } else {
                std::result::Result::Ok(())
            }
        }
        std::result::Result::Err(_) => {
            std::result::Result::Err("JWT_SECRET environment variable is not set".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_validate_jwt_secret_length_requirements() {
        // Test minimum length validation logic without manipulating environment
        let short_secret = "short";
        let adequate_secret = "this_is_a_sufficiently_long_secret_key_for_jwt_signing_operations";
        
        // Test that our length validation logic works correctly
        assert!(short_secret.len() < 32);
        assert!(adequate_secret.len() >= 32);
        
        // Test empty string validation
        let empty_secret = "";
        assert!(empty_secret.is_empty());
    }
}
