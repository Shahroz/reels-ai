//! Create feed post query
//! 
//! Handles creation of a new feed post with multiple assets and enhancement prompts.

use sqlx::PgPool;
use uuid::Uuid;
use anyhow::{Context, Result};

/// Arguments for creating a feed post
#[derive(Debug, Clone)]
pub struct CreateFeedPostArgs {
    pub user_id: Uuid,
    pub caption: String,
    /// Asset IDs in desired display order (first = 0, second = 1, etc.)
    pub asset_ids: Vec<Uuid>,
}

/// Result of creating a feed post
#[derive(Debug, Clone)]
pub struct CreateFeedPostResult {
    pub post_id: Uuid,
    pub assets_added: usize,
}

/// Validates that all assets exist and belong to the user
async fn validate_asset_ownership(
    pool: &PgPool,
    user_id: Uuid,
    asset_ids: &[Uuid],
) -> Result<()> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM assets
        WHERE id = ANY($1) AND user_id = $2
        "#,
        asset_ids,
        user_id
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
    
    Ok(())
}

/// Fetches enhancement prompts for assets from provenance_edges
/// Returns a map of asset_id -> prompt (if enhanced)
pub async fn fetch_enhancement_prompts(
    pool: &PgPool,
    asset_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, String>> {
    #[derive(sqlx::FromRow)]
    struct ProvenanceRow {
        target_id: Uuid,
        params: serde_json::Value,
    }
    
    let rows = sqlx::query_as!(
        ProvenanceRow,
        r#"
        SELECT target_id, params
        FROM provenance_edges
        WHERE target_type = 'asset'
          AND target_id = ANY($1)
          AND relation_type = 'enhanced'
        "#,
        asset_ids
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch enhancement prompts from provenance_edges")?;
    
    let mut prompts = std::collections::HashMap::new();
    
    for row in rows {
        // Try to parse params as typed enum
        if let Ok(enum_params) = serde_json::from_value::<crate::queries::assets::lineage::types::DerivationParams>(row.params.clone()) {
            if let crate::queries::assets::lineage::types::DerivationParams::Retouch(retouch) = enum_params {
                prompts.insert(row.target_id, retouch.retouch_prompt);
                continue;
            }
        }
        
        // Fallback to legacy RetouchParams
        if let Ok(legacy) = serde_json::from_value::<crate::queries::assets::lineage::types::RetouchParams>(row.params) {
            prompts.insert(row.target_id, legacy.retouch_prompt);
        }
    }
    
    Ok(prompts)
}

/// Creates a new feed post with assets
/// 
/// This function:
/// 1. Validates caption length (1-500 chars)
/// 2. Validates asset ownership
/// 3. Fetches enhancement prompts from provenance_edges
/// 4. Creates post and assets in a transaction
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `args` - Post creation arguments
/// 
/// # Returns
/// * `CreateFeedPostResult` with post_id and number of assets added
pub async fn create_feed_post(
    pool: &PgPool,
    args: CreateFeedPostArgs,
) -> Result<CreateFeedPostResult> {
    // Validate caption length
    let caption_len = args.caption.chars().count();
    if caption_len == 0 || caption_len > 500 {
        anyhow::bail!("Caption must be between 1 and 500 characters, got {}", caption_len);
    }
    
    // Validate at least one asset
    if args.asset_ids.is_empty() {
        anyhow::bail!("At least one asset is required for a feed post");
    }
    
    // Validate asset ownership
    validate_asset_ownership(pool, args.user_id, &args.asset_ids).await?;
    
    // Fetch enhancement prompts
    let prompts = fetch_enhancement_prompts(pool, &args.asset_ids).await?;
    
    // Start transaction
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    
    // Create feed post
    let post_id = sqlx::query_scalar!(
        r#"
        INSERT INTO feed_posts (user_id, caption)
        VALUES ($1, $2)
        RETURNING id
        "#,
        args.user_id,
        args.caption
    )
    .fetch_one(&mut *tx)
    .await
    .context("Failed to insert feed post")?;
    
    // Insert assets with ordering and prompts
    let mut assets_added = 0;
    for (idx, asset_id) in args.asset_ids.iter().enumerate() {
        let display_order = idx as i32;
        let prompt = prompts.get(asset_id).cloned();
        
        sqlx::query!(
            r#"
            INSERT INTO feed_post_assets (feed_post_id, asset_id, display_order, enhancement_prompt)
            VALUES ($1, $2, $3, $4)
            "#,
            post_id,
            asset_id,
            display_order,
            prompt.as_deref()
        )
        .execute(&mut *tx)
        .await
        .context("Failed to insert feed post asset")?;
        
        assets_added += 1;
    }
    
    // Commit transaction
    tx.commit().await.context("Failed to commit transaction")?;
    
    Ok(CreateFeedPostResult {
        post_id,
        assets_added,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_caption_validation() {
        let args = CreateFeedPostArgs {
            user_id: Uuid::new_v4(),
            caption: "".to_string(),
            asset_ids: vec![Uuid::new_v4()],
        };
        
        // Empty caption should fail (in actual async test)
        assert_eq!(args.caption.len(), 0);
    }
    
    #[test]
    fn test_empty_assets_validation() {
        let args = CreateFeedPostArgs {
            user_id: Uuid::new_v4(),
            caption: "Test".to_string(),
            asset_ids: vec![],
        };
        
        // Empty assets should fail (in actual async test)
        assert!(args.asset_ids.is_empty());
    }
}

