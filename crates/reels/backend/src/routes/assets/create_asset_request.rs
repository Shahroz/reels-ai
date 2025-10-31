//! Request body for creating a new asset.
//!
//! Defines the `CreateAssetRequest` with user-provided asset details.

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct CreateAssetRequest {
    #[schema(example = "image.png")]
    pub name: std::string::String,
    #[serde(rename = "type")]
    #[schema(example = "image/png")]
    pub r#type: std::string::String,
    #[schema(example = "https://storage.googleapis.com/bucket/image.png")]
    pub url: std::string::String,
    #[schema(example = "SGVsbG8gV29ybGQh")]
    pub content: std::string::String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub collection_id: Option<std::string::String>,
    #[schema(example = false, default = false, nullable = true)]
    pub is_public: Option<bool>,
}
