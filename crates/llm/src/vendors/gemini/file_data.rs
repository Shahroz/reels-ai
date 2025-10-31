//! Represents file data for the Gemini API File API.
//!
//! This struct contains the MIME type and file URI for files that have been
//! uploaded to the Gemini File API. It's used within the `Part` structure
//! as an alternative to `inline_data` for larger files.
//! Uses fully qualified paths for dependencies.
//! Used within the `Part` structure.

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct FileData {
    /// The MIME type of the file.
    pub mime_type: String,
    /// The URI of the file as returned by the File API.
    pub file_uri: String,
    /// The display name of the file (optional field from API response).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
} 