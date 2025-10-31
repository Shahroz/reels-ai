//! Video metadata for files uploaded to the Gemini File API.
//!
//! This struct contains metadata specific to video files, including duration.
//! Used as an optional field in the FileInfo struct when the uploaded file
//! is a video format. Uses fully qualified paths for dependencies.
//! Part of the Gemini File API response structure.

/// Video metadata for uploaded video files.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VideoMetadata {
    pub video_duration: String,
} 