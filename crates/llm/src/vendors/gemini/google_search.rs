//! Represents the configuration for a Google Search tool within the Gemini API.
//!
//! This struct is currently empty as per the API documentation at the time of writing.
//! It serves as a placeholder within the `Tool` struct.
//! Kept for potential future use if the API evolves.
//! Corresponds to the `googleSearch` field in the API request.

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct GoogleSearch {} // Empty struct as per docs
