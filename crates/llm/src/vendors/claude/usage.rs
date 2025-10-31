//! Defines the structure for token usage information in the Claude API response.
//!
//! This struct details the number of input and output tokens consumed by an API call.
//! May include additional fields related to caching based on API specifications.
//! Used within the `ClaudeMessageResponse` struct.
//! Supports deserialization from the API response JSON.

/// Represents the token usage information in the API response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    // Include cache tokens if needed based on API spec
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
    #[serde(default)]
    pub cache_read_input_tokens: u32,
}

// Basic struct definition, tests are typically part of response deserialization tests.
