//! Standard error response.
//!
//! Defines the `ErrorResponse` struct used for API error messages.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize}; // Already present implicitly via derive, but good to be explicit if needed elsewhere
use std::fmt;
use utoipa::ToSchema; // Already present implicitly via derive

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    #[schema(example = "Error message describing the issue")]
    pub error: String, // Changed to String from std::string::String for simplicity
}

// Implementation for std::fmt::Display is required for ResponseError.
impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

// Implementation for actix_web::ResponseError
impl ResponseError for ErrorResponse {
    fn status_code(&self) -> StatusCode {
        // We can't easily determine the status code from just the error string here.
        // Individual handlers should set the specific status code when creating HttpResponse.
        // This status_code is a fallback if ErrorResponse is returned directly as an Err.
        // For errors returned from `invite_member_handler` using `Ok(HttpResponse::BadRequest().json(ErrorResponse{...}))`,
        // the status code in `HttpResponse` takes precedence.
        // If this ErrorResponse were to be returned as `Err(ErrorResponse{...})` from a handler,
        // then this status_code method would be used. Defaulting to 500.
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self)
    }
}

// Optional: Convenience function to create an ErrorResponse from a string slice
impl From<&str> for ErrorResponse {
    fn from(s: &str) -> Self {
        ErrorResponse { error: s.to_string() }
    }
}

// Optional: Convenience function to create an ErrorResponse from a String
impl From<String> for ErrorResponse {
    fn from(s: String) -> Self {
        ErrorResponse { error: s }
    }
}