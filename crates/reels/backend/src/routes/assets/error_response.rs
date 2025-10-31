//! Standard error response.
//!
//! Defines the `ErrorResponse` struct used for API error messages.

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    #[schema(example = "Error message describing the issue")]
    pub error: std::string::String,
}
