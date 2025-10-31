//! Delete feed post query (soft delete)
//! 
//! Marks a feed post as deleted without removing it from the database.

use sqlx::PgPool;
use uuid::Uuid;
use anyhow::{Context, Result};

/// Soft deletes a feed post by setting deleted_at timestamp
/// 
/// Only the post owner can delete their post.
/// Post and its assets remain in database but won't appear in feeds.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `post_id` - UUID of the post to delete
/// * `user_id` - UUID of the requesting user (for authorization)
/// 
/// # Returns
/// * `Ok(true)` if post was deleted
/// * `Ok(false)` if post not found, already deleted, or user not authorized
/// * `Err` if database error
pub async fn delete_feed_post(
    pool: &PgPool,
    post_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let rows_affected = sqlx::query!(
        r#"
        UPDATE feed_posts
        SET deleted_at = NOW()
        WHERE id = $1
          AND user_id = $2
          AND deleted_at IS NULL
        "#,
        post_id,
        user_id
    )
    .execute(pool)
    .await
    .context("Failed to delete feed post")?
    .rows_affected();
    
    Ok(rows_affected > 0)
}

/// Hard deletes a feed post (admin only, use with caution)
/// 
/// Permanently removes post and all its assets from database.
/// This is CASCADE delete - assets in feed_post_assets will be removed automatically.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `post_id` - UUID of the post to delete permanently
/// 
/// # Returns
/// * `Ok(true)` if post was deleted
/// * `Ok(false)` if post not found
/// * `Err` if database error
#[allow(dead_code)]
pub async fn hard_delete_feed_post(
    pool: &PgPool,
    post_id: Uuid,
) -> Result<bool> {
    let rows_affected = sqlx::query!(
        r#"
        DELETE FROM feed_posts
        WHERE id = $1
        "#,
        post_id
    )
    .execute(pool)
    .await
    .context("Failed to hard delete feed post")?
    .rows_affected();
    
    Ok(rows_affected > 0)
}

