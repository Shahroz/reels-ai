//! Simple error response struct for HTTP error responses.
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(error: String) -> Self {
        Self { error }
    }
}

impl From<&str> for ErrorResponse {
    fn from(error: &str) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

impl From<String> for ErrorResponse {
    fn from(error: String) -> Self {
        Self { error }
    }
}

