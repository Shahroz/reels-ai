//! Update feed post query
//! 
//! Allows updating caption and assets of an existing feed post.

use sqlx::PgPool;
use uuid::Uuid;
use anyhow::{Context, Result};

/// Arguments for updating a feed post
#[derive(Debug, Clone)]
pub struct UpdateFeedPostArgs {
    pub post_id: Uuid,
    pub user_id: Uuid, // For authorization check
    pub caption: Option<String>, // None = don't update
    pub asset_ids: Option<Vec<Uuid>>, // None = don't update, Some = replace all assets
}

/// Updates a feed post
/// 
/// Only the post owner can update their post.
/// Can update caption and/or assets.
/// Assets are replaced entirely if provided (old assets removed, new ones added).
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `args` - Update arguments
/// 
/// # Returns
/// * `Ok(true)` if post was updated
/// * `Ok(false)` if post not found or user not authorized
/// * `Err` if validation fails or database error
pub async fn update_feed_post(
    pool: &PgPool,
    args: UpdateFeedPostArgs,
) -> Result<bool> {
    // Validate caption if provided
    if let Some(ref caption) = args.caption {
        let caption_len = caption.chars().count();
        if caption_len == 0 || caption_len > 500 {
            anyhow::bail!("Caption must be between 1 and 500 characters, got {}", caption_len);
        }
    }
    
    // Validate assets if provided
    if let Some(ref asset_ids) = args.asset_ids {
        if asset_ids.is_empty() {
            anyhow::bail!("At least one asset is required for a feed post");
        }
        
        // Validate asset ownership
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM assets
            WHERE id = ANY($1) AND user_id = $2
            "#,
            asset_ids,
            args.user_id
        )
        .fetch_one(pool)
        .await
        .context("Failed to validate asset ownership")?;
        
        let expected_count = asset_ids.len() as i64;
        if count != Some(expected_count) {
            anyhow::bail!(
                "Asset ownership validation failed: expected {} assets owned by user, found {}",
                expected_count,
                count.unwrap_or(0)
            );
        }
    }
    
    // Check if post exists and user owns it
    let post_exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM feed_posts
            WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
        )
        "#,
        args.post_id,
        args.user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to check post ownership")?
    .unwrap_or(false);
    
    if !post_exists {
        return Ok(false);
    }
    
    // Start transaction
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    
    // Update caption if provided
    if let Some(caption) = args.caption {
        sqlx::query!(
            r#"
            UPDATE feed_posts
            SET caption = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            caption,
            args.post_id
        )
        .execute(&mut *tx)
        .await
        .context("Failed to update caption")?;
    }
    
    // Update assets if provided
    if let Some(asset_ids) = args.asset_ids {
        // Delete old assets
        sqlx::query!(
            r#"
            DELETE FROM feed_post_assets
            WHERE feed_post_id = $1
            "#,
            args.post_id
        )
        .execute(&mut *tx)
        .await
        .context("Failed to delete old assets")?;
        
        // Fetch enhancement prompts for new assets
        let prompts = super::create_post::fetch_enhancement_prompts(pool, &asset_ids).await?;
        
        // Insert new assets
        for (idx, asset_id) in asset_ids.iter().enumerate() {
            let display_order = idx as i32;
            let prompt = prompts.get(asset_id).cloned();
            
            sqlx::query!(
                r#"
                INSERT INTO feed_post_assets (feed_post_id, asset_id, display_order, enhancement_prompt)
                VALUES ($1, $2, $3, $4)
                "#,
                args.post_id,
                asset_id,
                display_order,
                prompt.as_deref()
            )
            .execute(&mut *tx)
            .await
            .context("Failed to insert new asset")?;
        }
        
        // Update timestamp
        sqlx::query!(
            r#"
            UPDATE feed_posts
            SET updated_at = NOW()
            WHERE id = $1
            "#,
            args.post_id
        )
        .execute(&mut *tx)
        .await
        .context("Failed to update timestamp")?;
    }
    
    // Commit transaction
    tx.commit().await.context("Failed to commit transaction")?;
    
    Ok(true)
}

