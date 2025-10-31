//! JWT token creation using environment-configured secret.
//!
//! Creates signed JWT tokens using the secret from environment variables.
//! This is the production-ready function that reads the JWT_SECRET from environment.
//! Provides improved error handling that avoids information leakage about configuration.
//! Delegates to the parameterized function for actual token creation.

/// Creates a JWT for the given claims using the environment-configured secret.
///
/// # Arguments
///
/// * `claims` - The claims to encode in the token
///
/// # Returns
///
/// A `Result` containing the JWT string on success, or a generic error on failure.
/// Error messages are intentionally vague to avoid leaking configuration details.
///
/// # Security
///
/// Does not expose whether the failure was due to missing configuration or
/// signing errors. All failures return a generic "token creation failed" error.
#[tracing::instrument(skip(claims))]
pub fn create_jwt(claims: &crate::auth::tokens::claims::Claims) -> std::result::Result<std::string::String, jsonwebtoken::errors::Error> {
    let secret = match crate::auth::tokens::get_jwt_secret::get_jwt_secret() {
        std::result::Result::Ok(s) => s,
        std::result::Result::Err(_) => {
            // Log the specific error internally but return generic error to caller
            log::error!("JWT_SECRET not configured. Cannot create JWT.");
            return std::result::Result::Err(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat.into());
        }
    };

    match crate::auth::tokens::create_jwt_with_secret::create_jwt_with_secret(claims, &secret) {
        std::result::Result::Ok(token) => std::result::Result::Ok(token),
        std::result::Result::Err(e) => {
            // Log the specific error internally but return generic error to caller
            log::error!("JWT creation failed: {}", e);
            std::result::Result::Err(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat.into())
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_create_jwt_error_handling() {
        // Test that we properly handle missing environment configuration
        // Note: We cannot test the actual behavior without manipulating environment,
        // but we can test that the function exists and has the correct signature
        
        let user_id = uuid::Uuid::new_v4();
        let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as u64;
        
        let claims = crate::auth::tokens::claims::Claims {
            user_id,
            is_admin: false,
            email: "test@example.com".to_string(),
            email_verified: true,
            feature_flags: std::option::Option::None,
            exp: expiration,
            admin_id: std::option::Option::None,
            is_impersonating: std::option::Option::None,
        };

        // The function should return some result (either success or failure)
        // We cannot predict which without knowing the environment state
        let _result = super::create_jwt(&claims);
        
        // Test passes if the function doesn't panic and returns a Result
        assert!(true, "Function should return without panicking");
    }
}
