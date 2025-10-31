//! Represents a single candidate response within the Gemini API response structure.
//!
//! Each candidate contains the actual generated content from the model.
//! Typically, only the first candidate is used.
//! Uses fully qualified paths for dependencies.
//! Part of the `ApiResponse` structure.

#[derive(Debug, serde::Deserialize)]
pub struct Candidate {
    pub content: crate::vendors::gemini::content_response::ContentResponse,
}
