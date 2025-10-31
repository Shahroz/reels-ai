//! Response structure for Gemini File API upload operations.
//!
//! This struct represents the response returned by the Gemini File API after
//! successfully uploading a file. Contains the FileInfo for the uploaded file.
//! Used by the FileApiClient to handle upload responses.
//! Uses fully qualified paths for dependencies.

/// Response from file upload operation.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FileUploadResponse {
    pub file: crate::vendors::gemini::FileInfo,
} 