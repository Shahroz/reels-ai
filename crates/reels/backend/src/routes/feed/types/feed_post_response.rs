//! Response types for feed post data

/// Asset within a feed post
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct FeedPostAssetResponse {
    /// Asset UUID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub asset_id: uuid::Uuid,
    
    /// Full URL to the asset image
    #[schema(example = "https://storage.googleapis.com/...")]
    pub asset_url: String,
    
    /// Asset name/filename
    #[schema(example = "living-room-enhanced.jpg")]
    pub asset_name: String,
    
    /// Display order in carousel (0-indexed)
    #[schema(example = 0)]
    pub display_order: i32,
    
    /// AI enhancement prompt used for this asset (null for original images)
    #[schema(example = "make it brighter", nullable = true)]
    pub enhancement_prompt: Option<String>,
}

/// Complete feed post with all assets
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct FeedPostResponse {
    /// Post UUID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: uuid::Uuid,
    
    /// Author user UUID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid")]
    pub user_id: uuid::Uuid,
    
    /// Post caption
    #[schema(example = "Check out my beautiful enhanced living room!")]
    pub caption: String,
    
    /// Array of assets in display order
    pub assets: Vec<FeedPostAssetResponse>,
    
    /// Post creation timestamp
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Post last update timestamp
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::queries::feed::get_feed::FeedPostWithAssets> for FeedPostResponse {
    fn from(post: crate::queries::feed::get_feed::FeedPostWithAssets) -> Self {
        Self {
            id: post.id,
            user_id: post.user_id,
            caption: post.caption,
            assets: post.assets.into_iter().map(|a| FeedPostAssetResponse {
                asset_id: a.asset_id,
                asset_url: a.asset_url,
                asset_name: a.asset_name,
                display_order: a.display_order,
                enhancement_prompt: a.enhancement_prompt,
            }).collect(),
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}

