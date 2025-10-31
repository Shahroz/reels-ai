//! Service for saving assets from existing GCS URLs to the database.
//!
//! This module provides route-specific functionality for creating assets from existing
//! GCS URLs, mimicking the agent tool functionality but decoupled from the agent system.
//! Used primarily by the enhance_asset endpoint to save AI-enhanced images.

use crate::db::assets::Asset;
use sqlx::PgPool;

/// Data structure for assets to be saved from GCS URLs
#[derive(Debug, Clone)]
pub struct GcsAssetData {
    /// Name of the asset (e.g., 'enhanced_image_1.jpg')
    pub name: String,
    /// MIME type of the asset (e.g., 'image/jpeg', 'image/png')
    pub r#type: String,
    /// Full GCS URL of the existing asset
    pub gcs_url: String,
    /// GCS object name/path (e.g., 'user-id/asset-id.jpg')
    pub gcs_object_name: String,
    /// Optional collection ID to associate the asset with
    pub collection_id: Option<String>,
}

/// Saves multiple assets from existing GCS URLs to the database
/// 
/// This is a route-specific service that mimics the agent tool functionality
/// but is decoupled from the agent system. It creates new asset records in the
/// database for files that already exist in GCS.
/// 
/// # Arguments
/// 
/// * `pool` - Database connection pool
/// * `user_id` - ID of the user who will own the assets
/// * `assets_data` - Vector of asset data to save
/// * `track_rewards` - Whether to track credit reward progress for uploads
/// 
/// # Returns
/// 
/// A `Result` containing a vector of successfully created `Asset` objects,
/// or an error message if all assets failed to save.
/// 
/// # Behavior
/// 
/// - Partial success is allowed: if some assets save successfully and others fail,
///   the function returns the successful ones and logs warnings about failures
/// - Only returns an error if ALL assets fail to save
/// - Each asset gets a new UUID generated automatically
/// - Collection IDs are parsed from strings to UUIDs if provided
/// - If `track_rewards` is true, updates credit reward progress for each saved asset
/// 
/// # Examples
/// 
/// ```ignore
/// let assets_data = vec![
///     GcsAssetData {
///         name: "enhanced_image.jpg".to_string(),
///         r#type: "image/jpeg".to_string(),
///         gcs_url: "gs://my-bucket/enhanced/image.jpg".to_string(),
///         gcs_object_name: "user123/enhanced/image.jpg".to_string(),
///         collection_id: None,
///     }
/// ];
/// 
/// let saved_assets = save_assets_from_gcs_urls(&pool, user_id, assets_data, true).await?;
/// ```
pub async fn save_assets_from_gcs_urls(
    pool: &PgPool,
    user_id: uuid::Uuid,
    assets_data: Vec<GcsAssetData>,
    track_rewards: bool,
) -> Result<Vec<Asset>, String> {
    if assets_data.is_empty() {
        return Err("No assets provided to save".to_string());
    }

    let mut saved_assets = Vec::new();
    let mut errors = Vec::new();

    log::debug!("Attempting to save {} asset(s) for user {}", assets_data.len(), user_id);

    for (index, asset_data) in assets_data.iter().enumerate() {
        // Parse collection_id if provided
        let parsed_collection_id = asset_data.collection_id.as_ref().and_then(|id| {
            match uuid::Uuid::parse_str(id) {
                Ok(uuid) => Some(uuid),
                Err(e) => {
                    log::warn!("Invalid collection_id format '{}' for asset {}: {}", id, asset_data.name, e);
                    None
                }
            }
        });

        // Generate a new asset ID
        let asset_id = uuid::Uuid::new_v4();

        log::debug!("Creating asset {} with ID {} for user {}", asset_data.name, asset_id, user_id);

        // Insert asset metadata into the database
        // Note: This function saves assets from GCS URLs without downloading content,
        // so metadata extraction is not possible here
        let result = crate::queries::assets::create_asset::create_asset(
            pool,
            asset_id,
            Some(user_id),
            &asset_data.name,
            &asset_data.r#type,
            &asset_data.gcs_object_name,
            &asset_data.gcs_url,
            parsed_collection_id,
            None, // No metadata available from GCS URL saves
            false, // is_public - GCS imports are private by default
        )
        .await;

        match result {
            Ok(asset) => {
                log::debug!("Successfully saved asset '{}' with ID {}", asset_data.name, asset.id);
                saved_assets.push(asset);
            }
            Err(e) => {
                let error_msg = format!("Failed to save asset '{}' (index {}): {}", asset_data.name, index, e);
                log::error!("Error saving asset: {error_msg}");
                errors.push(error_msg);
            }
        }
    }

    // Determine response based on results
    let total_attempted = assets_data.len();
    let total_saved = saved_assets.len();
    let total_failed = errors.len();

    // Update credit reward progress for assets upload
    if let Err(e) = crate::queries::credit_rewards::update_user_reward_progress(
        &pool,
        user_id,
        crate::app_constants::credits_constants::CreditRewardActionTypes::UPLOAD_ASSETS,
        total_saved as i32,
    ).await {
        log::warn!("Failed to update credit reward progress for user {}: {}", user_id, e);
    }

    log::info!("Asset save operation completed for user {user_id}: {total_attempted} attempted, {total_saved} saved, {total_failed} failed");

    if total_failed > 0 && saved_assets.is_empty() {
        // All failed
        return Err(format!("Failed to save all {} assets. Errors: {}", total_attempted, errors.join("; ")));
    }

    // Update credit reward progress if tracking is enabled (bulk operation)
    if track_rewards && !saved_assets.is_empty() {
        if let Err(e) = crate::queries::credit_rewards::update_user_reward_progress(
            pool,
            user_id,
            crate::app_constants::credits_constants::CreditRewardActionTypes::UPLOAD_ASSETS,
            saved_assets.len() as i32,
        ).await {
            log::warn!("Failed to update credit reward progress for user {}: {}", user_id, e);
        }
    }

    // Log warnings for partial failures but still return successful assets
    if total_failed > 0 {
        log::warn!("Some assets failed to save for user {}: {}", user_id, errors.join("; "));
    }

    Ok(saved_assets)
}

/// Convenience function to save a single asset from GCS URL
/// 
/// # Arguments
/// 
/// * `pool` - Database connection pool
/// * `user_id` - ID of the user who will own the asset
/// * `asset_data` - Single asset data to save
/// * `track_rewards` - Whether to track credit reward progress for uploads
/// 
/// # Returns
/// 
/// A `Result` containing the created `Asset` object, or an error message.
pub async fn save_single_asset_from_gcs_url(
    pool: &PgPool,
    user_id: uuid::Uuid,
    asset_data: GcsAssetData,
    track_rewards: bool,
) -> Result<Asset, String> {
    let mut assets = save_assets_from_gcs_urls(pool, user_id, vec![asset_data], track_rewards).await?;
    
    assets.pop().ok_or_else(|| "Failed to save asset".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcs_asset_data_creation() {
        let asset_data = GcsAssetData {
            name: "test_image.jpg".to_string(),
            r#type: "image/jpeg".to_string(),
            gcs_url: "gs://my-bucket/test_image.jpg".to_string(),
            gcs_object_name: "user123/test_image.jpg".to_string(),
            collection_id: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
        };

        assert_eq!(asset_data.name, "test_image.jpg");
        assert_eq!(asset_data.r#type, "image/jpeg");
        assert_eq!(asset_data.gcs_url, "gs://my-bucket/test_image.jpg");
        assert_eq!(asset_data.gcs_object_name, "user123/test_image.jpg");
        assert!(asset_data.collection_id.is_some());
    }

    #[test]
    fn test_empty_assets_data() {
        // This would need a mock database to test properly
        // For now, just test the structure
        let empty_data: Vec<GcsAssetData> = vec![];
        assert!(empty_data.is_empty());
    }
} 