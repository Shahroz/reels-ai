//! Response type for feed list with pagination

use super::feed_post_response::FeedPostResponse;

/// Response for feed list endpoint with pagination metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct GetFeedResponse {
    /// Array of feed posts
    pub posts: Vec<FeedPostResponse>,
    
    /// Current page number (1-indexed)
    #[schema(example = 1)]
    pub page: i64,
    
    /// Total number of pages
    #[schema(example = 5)]
    pub total_pages: i64,
    
    /// Total number of posts in feed
    #[schema(example = 42)]
    pub total_count: i64,
}

