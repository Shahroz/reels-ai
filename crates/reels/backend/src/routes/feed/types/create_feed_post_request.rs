//! Request type for creating a feed post

/// Request body for creating a new feed post
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateFeedPostRequest {
    /// Caption for the post (required, 1-500 characters)
    #[schema(example = "Check out my beautiful enhanced living room!")]
    pub caption: String,
    
    /// Array of asset IDs to include in the post, in desired display order
    /// First asset will be display_order=0, second=1, etc.
    /// Must contain at least one asset ID.
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440000", "550e8400-e29b-41d4-a716-446655440001"]))]
    pub asset_ids: Vec<String>, // String UUIDs for JSON compatibility
}

