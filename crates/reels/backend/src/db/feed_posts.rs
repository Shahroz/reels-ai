// backend/src/db/feed_posts.rs
// Data models for `feed_posts` table

/// Represents a user-generated feed post.
///
/// A feed post is a user's publication to the public feed, containing a caption
/// and one or more associated assets (images). Posts can be soft-deleted via the
/// deleted_at field.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct FeedPost {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type = String)]
    pub user_id: uuid::Uuid,
    
    #[schema(example = "Check out my beautiful enhanced living room!")]
    pub caption: String,
    
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    
    #[schema(value_type = Option<String>, format = "date-time", example = "2024-04-21T10:00:00Z", nullable = true)]
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl FeedPost {
    /// Check if the post is deleted (soft delete)
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    
    /// Check if the post is active (not deleted)
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feed_post_is_deleted() {
        let mut post = FeedPost {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            caption: "Test caption".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };
        
        assert!(!post.is_deleted());
        assert!(post.is_active());
        
        post.deleted_at = Some(chrono::Utc::now());
        assert!(post.is_deleted());
        assert!(!post.is_active());
    }
}

