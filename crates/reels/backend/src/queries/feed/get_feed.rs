//! Get feed query with pagination
//! 
//! Fetches public feed posts with their assets in display order.

use sqlx::PgPool;
use anyhow::{Context, Result};

/// Asset within a feed post
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedAssetInfo {
    pub asset_id: uuid::Uuid,
    pub asset_url: String,
    pub asset_name: String,
    pub display_order: i32,
    pub enhancement_prompt: Option<String>,
}

/// Feed post with all its assets
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedPostWithAssets {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub caption: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub assets: Vec<FeedAssetInfo>,
}

/// Parameters for feed pagination
#[derive(Debug, Clone)]
pub struct GetFeedParams {
    pub page: i64,
    pub limit: i64,
}

impl Default for GetFeedParams {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
        }
    }
}

/// Result of get_feed query
#[derive(Debug, Clone)]
pub struct GetFeedResult {
    pub posts: Vec<FeedPostWithAssets>,
    pub total_count: i64,
    pub page: i64,
    pub total_pages: i64,
}

/// Fetches public feed with pagination
/// 
/// Returns active (non-deleted) posts in chronological order (newest first).
/// Each post includes all its assets with enhancement prompts.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `params` - Pagination parameters (page, limit)
/// 
/// # Returns
/// * `GetFeedResult` with posts and pagination metadata
pub async fn get_feed(
    pool: &PgPool,
    params: GetFeedParams,
) -> Result<GetFeedResult> {
    // Validate pagination params
    let page = params.page.max(1);
    let limit = params.limit.clamp(1, 100); // Max 100 posts per page
    let offset = (page - 1) * limit;
    
    // Get total count of active posts
    let total_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM feed_posts
        WHERE deleted_at IS NULL
        "#
    )
    .fetch_one(pool)
    .await
    .context("Failed to count total feed posts")?
    .unwrap_or(0);
    
    // Calculate total pages
    let total_pages = (total_count as f64 / limit as f64).ceil() as i64;
    
    // Fetch posts with pagination
    let post_rows = sqlx::query!(
        r#"
        SELECT id, user_id, caption, created_at, updated_at
        FROM feed_posts
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch feed posts")?;
    
    let mut posts = Vec::new();
    
    // For each post, fetch its assets
    for post_row in post_rows {
        let asset_rows = sqlx::query!(
            r#"
            SELECT
                fpa.asset_id,
                fpa.display_order,
                fpa.enhancement_prompt,
                a.url as asset_url,
                a.name as asset_name
            FROM feed_post_assets fpa
            JOIN assets a ON a.id = fpa.asset_id
            WHERE fpa.feed_post_id = $1
            ORDER BY fpa.display_order ASC
            "#,
            post_row.id
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch post assets")?;
        
        let assets: Vec<FeedAssetInfo> = asset_rows
            .into_iter()
            .map(|row| FeedAssetInfo {
                asset_id: row.asset_id,
                asset_url: row.asset_url,
                asset_name: row.asset_name,
                display_order: row.display_order,
                enhancement_prompt: row.enhancement_prompt,
            })
            .collect();
        
        posts.push(FeedPostWithAssets {
            id: post_row.id,
            user_id: post_row.user_id,
            caption: post_row.caption,
            created_at: post_row.created_at,
            updated_at: post_row.updated_at,
            assets,
        });
    }
    
    Ok(GetFeedResult {
        posts,
        total_count,
        page,
        total_pages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pagination_params_default() {
        let params = GetFeedParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.limit, 20);
    }
    
    #[test]
    fn test_pagination_params_clamping() {
        let params = GetFeedParams {
            page: 0,
            limit: 200,
        };
        
        let page = params.page.max(1);
        let limit = params.limit.clamp(1, 100);
        
        assert_eq!(page, 1);
        assert_eq!(limit, 100);
    }
}

