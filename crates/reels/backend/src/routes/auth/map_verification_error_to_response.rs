//! Maps core verification errors to HTTP responses.
//!
//! Provides a centralized function to convert `VerificationError` enum values
//! into appropriate HTTP responses with user-friendly error messages and
//! correct status codes.

/// Maps a verification error to an appropriate HTTP response.
///
/// This function centralizes error handling for magic link verification,
/// ensuring consistent error messages and status codes.
///
/// # Error Status Codes
///
/// - `InvalidToken` → 400 Bad Request (malformed or expired token)
/// - `UserNotFound` → 401 Unauthorized (authentication failed)
/// - `TokenAlreadyUsed` → 401 Unauthorized (single-use enforcement)
/// - `SessionCreationFailed` → 500 Internal Server Error (server issue)
///
/// # Arguments
///
/// * `error` - The verification error to map
///
/// # Returns
///
/// An `HttpResponse` with appropriate status code and JSON error message
pub fn map_verification_error_to_response(
    error: crate::routes::auth::verify_magic_link_core::VerificationError,
) -> actix_web::HttpResponse {
    use crate::routes::auth::verify_magic_link_core::VerificationError;
    
    match error {
        VerificationError::InvalidToken => {
            actix_web::HttpResponse::BadRequest()
                .json(serde_json::json!({
                    "error": "Invalid or expired token"
                }))
        }
        VerificationError::UserNotFound => {
            actix_web::HttpResponse::Unauthorized()
                .json(serde_json::json!({
                    "error": "Authentication failed"
                }))
        }
        VerificationError::TokenAlreadyUsed => {
            actix_web::HttpResponse::Unauthorized()
                .json(serde_json::json!({
                    "error": "This login link has already been used or is no longer valid"
                }))
        }
        VerificationError::SessionCreationFailed => {
            actix_web::HttpResponse::InternalServerError()
                .json(serde_json::json!({
                    "error": "Authentication failed"
                }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_token_returns_400() {
        use crate::routes::auth::verify_magic_link_core::VerificationError;
        let response = map_verification_error_to_response(VerificationError::InvalidToken);
        assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_user_not_found_returns_401() {
        use crate::routes::auth::verify_magic_link_core::VerificationError;
        let response = map_verification_error_to_response(VerificationError::UserNotFound);
        assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_token_already_used_returns_401() {
        use crate::routes::auth::verify_magic_link_core::VerificationError;
        let response = map_verification_error_to_response(VerificationError::TokenAlreadyUsed);
        assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_session_creation_failed_returns_500() {
        use crate::routes::auth::verify_magic_link_core::VerificationError;
        let response = map_verification_error_to_response(VerificationError::SessionCreationFailed);
        assert_eq!(response.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    }
}

