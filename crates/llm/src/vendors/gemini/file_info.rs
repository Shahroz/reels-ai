//! Represents file information returned by the Gemini File API.
//!
//! This struct contains metadata about files uploaded to the Gemini File API,
//! including file name, MIME type, size, creation time, and processing state.
//! All fields use `#[serde(default)]` to handle inconsistent API responses.
//! Used by the FileApiClient for file management operations.

/// Represents a file uploaded to the Gemini File API.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FileInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub size_bytes: String,
    #[serde(default)]
    pub create_time: String,
    #[serde(default)]
    pub update_time: String,
    #[serde(default)]
    pub expiration_time: String,
    #[serde(default)]
    pub sha256_hash: String,
    #[serde(default)]
    pub uri: String,
    #[serde(default)]
    pub state: String,
    pub video_metadata: Option<crate::vendors::gemini::VideoMetadata>,
} 