//! Request structure for confirming asset uploads.
//!
//! Defines the structure used to confirm that an asset has been
//! successfully uploaded to Google Cloud Storage and should be
//! registered in the database.

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct ConfirmUploadRequest {
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub asset_id: uuid::Uuid,
    #[schema(example = "property-video.mp4")]
    pub file_name: std::string::String,
    #[schema(example = "video/mp4")]
    pub content_type: std::string::String,
    #[schema(example = false, default = false, nullable = true)]
    pub is_public: Option<bool>,
} 