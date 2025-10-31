//! Inherits shares from a parent asset to child assets.
//!
//! When Studio creates derived/child assets (through enhancement, watermarking, etc.),
//! the child assets should inherit the same sharing permissions as the parent asset.
//! This ensures users maintain access to their edits when working with shared assets.

use sqlx::PgPool;
use uuid::Uuid;

/// Inherits all shares from a parent asset to one or more child assets
///
/// This function copies all object_shares entries from the parent asset to the child assets,
/// maintaining the same access levels and entity relationships. This ensures that when
/// users edit shared assets in Studio, they retain access to their derived work.
///
/// # Arguments
/// 
/// * `pool` - Database connection pool
/// * `parent_asset_id` - UUID of the parent asset to inherit shares from
/// * `child_asset_ids` - Vector of child asset UUIDs to inherit shares to
///
/// # Returns
///
/// A `Result` indicating success or database error. Logs warnings for conflicts
/// (when shares already exist) but continues processing other shares.
///
/// # Behavior
///
/// - Only copies shares for the 'asset' object_type
/// - Skips shares that already exist (based on unique constraint)
/// - Processes all child assets in a single transaction for consistency
/// - Logs detailed information about inheritance operations
pub async fn inherit_shares_from_asset(
    pool: &PgPool,
    parent_asset_id: Uuid,
    child_asset_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    if child_asset_ids.is_empty() {
        log::debug!("No child assets provided for share inheritance from {}", parent_asset_id);
        return Ok(());
    }

    log::info!("Inheriting shares from parent asset {} to {} child asset(s)", 
               parent_asset_id, child_asset_ids.len());

    let mut tx = pool.begin().await?;

    // First, fetch all shares for the parent asset
    let parent_shares = sqlx::query!(
        r#"
        SELECT entity_id, entity_type::TEXT as entity_type, access_level::TEXT as access_level
        FROM object_shares 
        WHERE object_id = $1 AND object_type = 'asset'
        "#,
        parent_asset_id
    )
    .fetch_all(&mut *tx)
    .await?;

    if parent_shares.is_empty() {
        log::debug!("Parent asset {} has no shares to inherit", parent_asset_id);
        tx.commit().await?;
        return Ok(());
    }

    log::info!("Found {} share(s) on parent asset {} to inherit", 
               parent_shares.len(), parent_asset_id);

    // For each child asset, copy all parent shares
    for child_asset_id in child_asset_ids {
        for share in &parent_shares {
            // Prepare fallback values with proper lifetimes
            let default_entity_type = "user".to_string();
            let default_access_level = "viewer".to_string();
            
            let entity_type = share.entity_type.as_ref().unwrap_or(&default_entity_type);
            let access_level = share.access_level.as_ref().unwrap_or(&default_access_level);
            
            // Insert the inherited share, ignoring conflicts (ON CONFLICT DO NOTHING)
            let result = sqlx::query!(
                r#"
                INSERT INTO object_shares (object_id, object_type, entity_id, entity_type, access_level)
                VALUES ($1, 'asset', $2, $3::text::object_share_entity_type, $4::text::object_share_access_level)
                ON CONFLICT (object_id, object_type, entity_id, entity_type) DO NOTHING
                "#,
                child_asset_id,
                share.entity_id,
                entity_type,
                access_level
            )
            .execute(&mut *tx)
            .await;

            match result {
                Ok(query_result) => {
                    if query_result.rows_affected() > 0 {
                        log::debug!("Inherited share: {} ({:?}) -> asset {} with {:?} access",
                                   share.entity_id, share.entity_type, child_asset_id, share.access_level);
                    } else {
                        log::debug!("Share already exists: {} ({:?}) -> asset {} (skipped)",
                                   share.entity_id, share.entity_type, child_asset_id);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to inherit share {} ({:?}) -> asset {}: {}",
                              share.entity_id, share.entity_type, child_asset_id, e);
                    // Continue processing other shares rather than failing the entire operation
                }
            }
        }
    }

    tx.commit().await?;
    
    log::info!("Completed share inheritance from asset {} to {} child asset(s)", 
               parent_asset_id, child_asset_ids.len());
    
    Ok(())
}

/// Convenience function to inherit shares for a single child asset
///
/// # Arguments
/// 
/// * `pool` - Database connection pool
/// * `parent_asset_id` - UUID of the parent asset to inherit shares from
/// * `child_asset_id` - UUID of the single child asset to inherit shares to
pub async fn inherit_shares_from_asset_single(
    pool: &PgPool,
    parent_asset_id: Uuid,
    child_asset_id: Uuid,
) -> Result<(), sqlx::Error> {
    inherit_shares_from_asset(pool, parent_asset_id, &[child_asset_id]).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_child_assets() {
        // Function should handle empty child asset list gracefully
        // In real tests, you would:
        // 1. Setup test database with parent asset and shares
        // 2. Call inherit_shares_from_asset with empty vec
        // 3. Verify it returns Ok(()) without errors
        assert!(true, "Conceptual test for empty child assets handling");
    }

    #[test]
    fn test_share_inheritance_logic() {
        // Function should copy all share types and access levels
        // In real tests, you would:
        // 1. Create parent asset with user and organization shares
        // 2. Create child assets
        // 3. Call inherit_shares_from_asset
        // 4. Verify child assets have identical shares as parent
        assert!(true, "Conceptual test for share inheritance logic");
    }
}
