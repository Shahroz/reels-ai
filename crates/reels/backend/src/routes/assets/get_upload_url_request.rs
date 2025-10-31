//! Request structure for generating a signed upload URL for asset uploads.
//!
//! Defines the structure with file metadata needed to generate
//! a signed URL for direct client uploads to Google Cloud Storage.

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct GetUploadUrlRequest {
    #[schema(example = "property-video.mp4")]
    pub file_name: std::string::String,
    #[schema(example = 104857600)]
    pub file_size: u64,
    #[serde(rename = "contentType")]
    #[schema(example = "video/mp4")]
    pub content_type: std::string::String,
} 