// backend/src/db/feed_post_assets.rs
// Data models for `feed_post_assets` table

/// Represents an asset within a feed post.
///
/// This is a junction table linking feed posts to assets, with additional
/// metadata like display order and the enhancement prompt used for this specific asset.
/// Supports multi-image posts with ordering and prompt tracking.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct FeedPostAsset {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type = String)]
    pub feed_post_id: uuid::Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440002", format = "uuid", value_type = String)]
    pub asset_id: uuid::Uuid,
    
    #[schema(example = 0)]
    pub display_order: i32,
    
    #[schema(example = "make it brighter", nullable = true)]
    pub enhancement_prompt: Option<String>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl FeedPostAsset {
    /// Check if this asset has an enhancement prompt (i.e., was AI-enhanced)
    pub fn is_enhanced(&self) -> bool {
        self.enhancement_prompt.is_some()
    }
    
    /// Check if this is an original (non-enhanced) asset
    pub fn is_original(&self) -> bool {
        self.enhancement_prompt.is_none()
    }
    
    /// Get the prompt text or a default message
    pub fn prompt_or_default(&self) -> String {
        self.enhancement_prompt.clone().unwrap_or_else(|| "Original - no enhancements applied".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feed_post_asset_is_enhanced() {
        let mut asset = FeedPostAsset {
            id: uuid::Uuid::new_v4(),
            feed_post_id: uuid::Uuid::new_v4(),
            asset_id: uuid::Uuid::new_v4(),
            display_order: 0,
            enhancement_prompt: None,
            created_at: chrono::Utc::now(),
        };
        
        assert!(asset.is_original());
        assert!(!asset.is_enhanced());
        assert_eq!(asset.prompt_or_default(), "Original - no enhancements applied");
        
        asset.enhancement_prompt = Some("make it brighter".to_string());
        assert!(asset.is_enhanced());
        assert!(!asset.is_original());
        assert_eq!(asset.prompt_or_default(), "make it brighter");
    }
}

