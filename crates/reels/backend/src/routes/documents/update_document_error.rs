//! Error handling for document update operations.
//!
//! This module defines structured error types and HTTP response mapping
//! for document update failures. It provides clear error categorization
//! and appropriate HTTP status code mapping for API consumers.

/// Structured error types for document update operations.
///
/// This enum provides comprehensive error handling for document updates,
/// categorizing failures by type for appropriate HTTP response mapping.
/// Each variant includes contextual error messages for debugging.
#[derive(std::fmt::Debug)]
pub enum UpdateDocumentError {
    ValidationError(std::string::String),
    PermissionDenied(std::string::String),
    NotFound(std::string::String),
    Forbidden(std::string::String),
    DatabaseError(std::string::String),
}

impl UpdateDocumentError {
    /// Converts the error into an appropriate HTTP response.
    ///
    /// Maps error types to standard HTTP status codes and formats
    /// error messages for API consumers. Database errors are logged
    /// but sanitized to prevent information leakage.
    pub fn to_http_response(self) -> actix_web::HttpResponse {
        match self {
            UpdateDocumentError::ValidationError(msg) => {
                actix_web::HttpResponse::UnprocessableEntity().json(crate::routes::error_response::ErrorResponse { error: msg })
            }
            UpdateDocumentError::PermissionDenied(msg) | UpdateDocumentError::NotFound(msg) => {
                actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse { error: msg })
            }
            UpdateDocumentError::Forbidden(msg) => {
                actix_web::HttpResponse::Forbidden().json(crate::routes::error_response::ErrorResponse { error: msg })
            }
            UpdateDocumentError::DatabaseError(msg) => {
                log::error!("Database error: {msg}");
                actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse { 
                    error: "Internal server error".to_string() 
                })
            }
        }
    }
}

// Note: This module contains simple error type to HTTP status mappings.
// These mappings are straightforward and are thoroughly tested via integration tests
// rather than unit tests that would essentially test `assert_eq!(FORBIDDEN, FORBIDDEN)`. 