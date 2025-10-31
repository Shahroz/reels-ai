//! JWT token creation with provided secret for testing and dependency injection.
//!
//! Creates signed JWT tokens using a provided secret key, enabling testing and
//! dependency injection patterns. Uses HMAC-SHA256 algorithm for signing.
//! This function is the core JWT creation logic that other functions delegate to.
//! Designed to be pure and testable without environment dependencies.

/// Creates a JWT for the given claims using a provided secret key.
///
/// # Arguments
///
/// * `claims` - The claims to encode in the token
/// * `secret` - The secret key to use for signing
///
/// # Returns
///
/// A `Result` containing the JWT string on success, or a signing error on failure.
///
/// # Security
///
/// Uses HMAC-SHA256 algorithm. The secret should be cryptographically strong.
/// This function does not validate the secret strength - that should be done
/// during application startup.
#[tracing::instrument(skip(claims, secret))]
pub fn create_jwt_with_secret(claims: &crate::auth::tokens::claims::Claims, secret: &str) -> std::result::Result<std::string::String, jsonwebtoken::errors::Error> {
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
    )?;

    std::result::Result::Ok(token)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_create_jwt_basic_functionality() {
        let test_secret = "test_secret_with_sufficient_length_for_hmac_sha256_algorithm";
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

        let result = super::create_jwt_with_secret(&claims, test_secret);
        assert!(result.is_ok(), "JWT creation should succeed with valid inputs");
        
        let token = result.unwrap();
        assert!(!token.is_empty(), "Token should not be empty");
        
        // Basic JWT format check (should have 3 parts separated by dots)
        let parts: std::vec::Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have exactly 3 parts");
    }

    #[test]
    fn test_create_jwt_with_impersonation() {
        let test_secret = "another_test_secret_with_adequate_length_for_testing_purposes";
        let user_id = uuid::Uuid::new_v4();
        let admin_id = uuid::Uuid::new_v4();
        let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as u64;
        
        let claims = crate::auth::tokens::claims::Claims {
            user_id,
            is_admin: true,
            email: "admin@example.com".to_string(),
            email_verified: true,
            feature_flags: std::option::Option::Some(std::vec!["admin_panel".to_string()]),
            exp: expiration,
            admin_id: std::option::Option::Some(admin_id),
            is_impersonating: std::option::Option::Some(true),
        };

        let result = super::create_jwt_with_secret(&claims, test_secret);
        assert!(result.is_ok(), "JWT creation should succeed with impersonation claims");
    }
}
