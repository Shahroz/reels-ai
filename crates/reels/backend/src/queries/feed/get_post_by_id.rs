//! Get single feed post by ID
//! 
//! Fetches a specific feed post with all its assets.

use sqlx::PgPool;
use uuid::Uuid;
use anyhow::{Context, Result};

/// Re-use types from get_feed
pub use super::get_feed::{FeedAssetInfo, FeedPostWithAssets};

/// Fetches a single feed post by ID
/// 
/// Returns the post with all its assets in display order.
/// Only returns active (non-deleted) posts.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `post_id` - UUID of the post to fetch
/// 
/// # Returns
/// * `Some(FeedPostWithAssets)` if found and active
/// * `None` if not found or deleted
pub async fn get_feed_post_by_id(
    pool: &PgPool,
    post_id: Uuid,
) -> Result<Option<FeedPostWithAssets>> {
    // Fetch post (only if not deleted)
    let post_row = sqlx::query!(
        r#"
        SELECT id, user_id, caption, created_at, updated_at
        FROM feed_posts
        WHERE id = $1 AND deleted_at IS NULL
        "#,
        post_id
    )
    .fetch_optional(pool)
    .await
    .context("Failed to fetch feed post")?;
    
    let Some(post_row) = post_row else {
        return Ok(None);
    };
    
    // Fetch assets for this post
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
        post_id
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
    
    Ok(Some(FeedPostWithAssets {
        id: post_row.id,
        user_id: post_row.user_id,
        caption: post_row.caption,
        created_at: post_row.created_at,
        updated_at: post_row.updated_at,
        assets,
    }))
}

