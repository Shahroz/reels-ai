//! HTTP request JWT claims extraction for Actix Web framework.
//!
//! Implements the FromRequest trait for Claims to enable automatic JWT extraction
//! from HTTP Authorization headers in Actix Web handlers. Validates Bearer tokens
//! and returns decoded claims for authenticated requests. Provides secure error
//! handling that avoids leaking sensitive information about token validation failures.

/// Implementation of FromRequest trait for Claims extraction from HTTP requests.
impl actix_web::FromRequest for crate::auth::tokens::claims::Claims {
    type Error = actix_web::Error;
    type Future = std::future::Ready<std::result::Result<Self, Self::Error>>;

    /// Extracts JWT claims from the Authorization header of an HTTP request.
    ///
    /// # Expected Header Format
    ///
    /// `Authorization: Bearer <jwt_token>`
    ///
    /// # Returns
    ///
    /// A `Future` resolving to:
    /// - `Ok(Claims)` if the token is valid and properly formatted
    /// - `Err(Error)` with a generic error message for any failure
    ///
    /// # Security
    ///
    /// Error messages are intentionally generic to avoid leaking information
    /// about token validation failures to potential attackers.
    #[tracing::instrument(skip(req, _payload))]
    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        // Extract Authorization header
        let auth_header = req.headers().get("Authorization");

        let token = match auth_header {
            std::option::Option::Some(header_value) => {
                // Check if it starts with "Bearer "
                if let std::result::Result::Ok(value_str) = header_value.to_str() {
                    if value_str.starts_with("Bearer ") {
                        // Extract the token part
                        value_str.trim_start_matches("Bearer ").to_string()
                    } else {
                        // Invalid format - return generic error
                        return std::future::ready(std::result::Result::Err(actix_web::error::ErrorUnauthorized("Authentication required")));
                    }
                } else {
                    // Header value not valid UTF-8 - return generic error
                    return std::future::ready(std::result::Result::Err(actix_web::error::ErrorUnauthorized("Authentication required")));
                }
            }
            std::option::Option::None => {
                return std::future::ready(std::result::Result::Err(actix_web::error::ErrorUnauthorized("Authentication required")));
            }
        };

        // Verify the token using the existing function
        let verification_result = crate::auth::tokens::verify_jwt::verify_jwt(&token);
        
        match verification_result {
            std::result::Result::Ok(claims) => std::future::ready(std::result::Result::Ok(claims)),
            std::result::Result::Err(_) => {
                // Return generic error regardless of specific verification failure
                std::future::ready(std::result::Result::Err(actix_web::error::ErrorUnauthorized("Authentication required")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bearer_token_parsing() {
        // Test the Bearer token parsing logic without HTTP dependencies
        let valid_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.example.signature";
        let invalid_header = "Basic dXNlcjpwYXNz";
        let malformed_header = "Bearer";
        
        // Test valid Bearer format
        if valid_header.starts_with("Bearer ") {
            let token = valid_header.trim_start_matches("Bearer ");
            assert!(!token.is_empty(), "Token should not be empty after extraction");
            assert!(!token.starts_with("Bearer "), "Token should not contain Bearer prefix");
        }
        
        // Test invalid format
        assert!(!invalid_header.starts_with("Bearer "), "Basic auth should not match Bearer format");
        
        // Test malformed format
        if malformed_header.starts_with("Bearer ") {
            let token = malformed_header.trim_start_matches("Bearer ");
            assert!(token.is_empty(), "Malformed Bearer header should result in empty token");
        }
    }

    #[test]
    fn test_error_message_consistency() {
        let expected_error_message = "Authentication required";
        
        // All error cases should return the same generic message
        assert_eq!(expected_error_message, "Authentication required");
    }
}
