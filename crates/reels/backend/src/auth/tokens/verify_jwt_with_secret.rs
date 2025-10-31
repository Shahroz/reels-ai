//! JWT token verification with provided secret for testing and dependency injection.
//!
//! Verifies JWT tokens using a provided secret key, enabling testing and
//! dependency injection patterns. Validates signature, expiration, and format.
//! Includes clock skew tolerance for production reliability.
//! This is the core verification logic that other functions delegate to.

/// Verifies a JWT and returns the claims if valid using a provided secret.
///
/// # Arguments
///
/// * `token` - The JWT string to verify
/// * `secret` - The secret key to use for verification
///
/// # Returns
///
/// A `Result` containing the decoded `Claims` on success, or a verification error on failure.
///
/// # Security
///
/// - Validates HMAC-SHA256 signature
/// - Checks token expiration with 5-second clock skew tolerance
/// - Validates token structure and format
#[tracing::instrument(skip(token, secret))]
pub fn verify_jwt_with_secret(token: &str, secret: &str) -> std::result::Result<crate::auth::tokens::claims::Claims, jsonwebtoken::errors::Error> {
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.leeway = 5; // Add 5 seconds leeway for clock skew

    let token_data = jsonwebtoken::decode::<crate::auth::tokens::claims::Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )?;

    std::result::Result::Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_verify_jwt_round_trip() {
        let test_secret = "test_secret_for_verification_round_trip_testing_with_adequate_length";
        let user_id = uuid::Uuid::new_v4();
        let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as u64;
        
        let original_claims = crate::auth::tokens::claims::Claims {
            user_id,
            is_admin: true,
            email: "test@example.com".to_string(),
            email_verified: true,
            feature_flags: std::option::Option::Some(std::vec!["feature1".to_string()]),
            exp: expiration,
            admin_id: std::option::Option::None,
            is_impersonating: std::option::Option::None,
        };

        // Create a token using our creation function
        let token = crate::auth::tokens::create_jwt_with_secret::create_jwt_with_secret(&original_claims, test_secret)
            .expect("Token creation should succeed");

        // Verify the token using our verification function
        let verified_claims = super::verify_jwt_with_secret(&token, test_secret)
            .expect("Token verification should succeed");

        // Claims should match exactly
        assert_eq!(verified_claims, original_claims);
    }

    #[test]
    fn test_verify_jwt_with_wrong_secret() {
        let correct_secret = "correct_secret_for_signing_with_adequate_length_for_hmac";
        let wrong_secret = "wrong_secret_for_verification_with_adequate_length_for_hmac";
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

        // Create token with correct secret
        let token = crate::auth::tokens::create_jwt_with_secret::create_jwt_with_secret(&claims, correct_secret)
            .expect("Token creation should succeed");

        // Verify with wrong secret should fail
        let result = super::verify_jwt_with_secret(&token, wrong_secret);
        assert!(result.is_err(), "Verification should fail with wrong secret");
        
        match result.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                // Expected error type
            }
            other => panic!("Expected InvalidSignature error, got: {:?}", other),
        }
    }

    #[test]
    fn test_verify_expired_jwt() {
        let test_secret = "test_secret_for_expired_token_testing_with_adequate_length";
        let user_id = uuid::Uuid::new_v4();
        let expired_time = (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as u64;
        
        let expired_claims = crate::auth::tokens::claims::Claims {
            user_id,
            is_admin: false,
            email: "test@example.com".to_string(),
            email_verified: true,
            feature_flags: std::option::Option::None,
            exp: expired_time,
            admin_id: std::option::Option::None,
            is_impersonating: std::option::Option::None,
        };

        // Create token with expired timestamp
        let token = crate::auth::tokens::create_jwt_with_secret::create_jwt_with_secret(&expired_claims, test_secret)
            .expect("Token creation should succeed even with expired timestamp");

        // Verification should fail due to expiration
        let result = super::verify_jwt_with_secret(&token, test_secret);
        assert!(result.is_err(), "Verification should fail for expired token");
        
        match result.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                // Expected error type
            }
            other => panic!("Expected ExpiredSignature error, got: {:?}", other),
        }
    }
}
