//! Request type for updating a feed post

/// Request body for updating an existing feed post
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateFeedPostRequest {
    /// New caption (optional, if provided must be 1-500 characters)
    #[schema(example = "Updated caption for my post", nullable = true)]
    pub caption: Option<String>,
    
    /// New array of asset IDs (optional)
    /// If provided, completely replaces existing assets
    /// Must contain at least one asset ID if provided
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440000"]), nullable = true)]
    pub asset_ids: Option<Vec<String>>, // String UUIDs for JSON compatibility
}

