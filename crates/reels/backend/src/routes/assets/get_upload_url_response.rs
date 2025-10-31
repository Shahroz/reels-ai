//! Response structure for generating a signed upload URL for asset uploads.
//!
//! Defines the structure returned when a signed upload URL has been
//! successfully generated for direct client uploads to Google Cloud Storage.

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct GetUploadUrlResponse {
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub asset_id: uuid::Uuid,
    #[schema(example = "https://storage.googleapis.com/bucket/upload?signed_url_params")]
    pub upload_url: std::string::String,
    #[schema(example = "PUT")]
    pub upload_method: std::string::String,
    #[schema(example = "2024-01-01T12:15:00Z")]
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[schema(example = "user123/asset456.mp4")]
    pub object_name: std::string::String,
} 