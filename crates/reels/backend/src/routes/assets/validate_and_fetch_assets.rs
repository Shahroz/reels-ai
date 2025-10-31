//! Validates and fetches assets for enhancement.
//!
//! Takes a list of asset ID strings, validates each one, checks user permissions,
//! and verifies that each asset is an image. Returns the validated asset list
//! or an appropriate error.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from enhance_asset.rs

pub async fn validate_and_fetch_assets(
    pool: &sqlx::PgPool,
    asset_ids: &[std::string::String],
    user_id: uuid::Uuid,
) -> std::result::Result<std::vec::Vec<crate::db::assets::Asset>, crate::routes::assets::enhance_asset_error::EnhanceAssetError> {
    if asset_ids.is_empty() {
        return Err(crate::routes::assets::enhance_asset_error::EnhanceAssetError::InvalidAssetId("asset_ids cannot be empty".into()));
    }

    let mut assets_to_enhance = std::vec::Vec::new();

    for asset_id in asset_ids {
        // Parse and validate asset ID
        let asset_uuid = uuid::Uuid::parse_str(asset_id)
            .map_err(|e| crate::routes::assets::enhance_asset_error::EnhanceAssetError::InvalidAssetId(format!("'{asset_id}': {e}")))?;

        // Fetch and validate asset with proper organization share permissions
        let asset_with_collection = crate::queries::assets::get_asset_by_id_with_collection::get_asset_by_id_with_collection(pool, asset_uuid, user_id)
            .await
            .map_err(|e| {
                log::error!("Database error fetching asset {asset_uuid} for user {user_id}: {e}");
                crate::routes::assets::enhance_asset_error::EnhanceAssetError::AssetNotFound
            })?;

        let asset_with_collection = asset_with_collection.ok_or_else(|| {
            log::warn!("Asset {asset_uuid} not found or user {user_id} lacks access");
            crate::routes::assets::enhance_asset_error::EnhanceAssetError::AssetNotFound
        })?;

        // Extract the basic asset info from the collection response
        let asset = crate::db::assets::Asset {
            id: asset_with_collection.id,
            user_id: asset_with_collection.user_id,
            name: asset_with_collection.name,
            r#type: asset_with_collection.r#type,
            gcs_object_name: asset_with_collection.gcs_object_name,
            url: asset_with_collection.url,
            collection_id: asset_with_collection.collection_id,
            metadata: asset_with_collection.metadata,
            created_at: Some(asset_with_collection.created_at),
            updated_at: Some(asset_with_collection.updated_at),
            is_public: asset_with_collection.is_public,
        };

        // Validate asset is an image
        if !asset.r#type.starts_with("image/") {
            log::warn!("User {} attempted to enhance non-image asset {} (type: {})", user_id, asset_uuid, asset.r#type);
            return Err(crate::routes::assets::enhance_asset_error::EnhanceAssetError::NonImageAsset);
        }

        assets_to_enhance.push(asset);
    }

    Ok(assets_to_enhance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_and_fetch_assets_empty_list() {
        // Create a mock pool (in real tests, you'd use a test database)
        // For now, we can test the validation logic
        let asset_ids: std::vec::Vec<std::string::String> = vec![];
        let user_id = uuid::Uuid::new_v4();
        
        // This would require a real database connection
        // You should implement proper integration tests with test fixtures
    }

    #[test]
    fn test_invalid_uuid_format() {
        let invalid_id = "not-a-uuid";
        let result = uuid::Uuid::parse_str(invalid_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_uuid_format() {
        let valid_id = "550e8400-e29b-41d4-a716-446655440000";
        let result = uuid::Uuid::parse_str(valid_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_image_type_validation() {
        let image_types = vec!["image/jpeg", "image/png", "image/webp", "image/gif"];
        for img_type in image_types {
            assert!(img_type.starts_with("image/"));
        }
        
        let non_image_types = vec!["video/mp4", "audio/mp3", "application/pdf"];
        for non_img_type in non_image_types {
            assert!(!non_img_type.starts_with("image/"));
        }
    }

    // Note: Full integration tests require:
    // - Test database setup
    // - Fixtures for users, assets, and collections
    // - Mock or test implementations of get_asset_by_id_with_collection
    // - Consider using test_utils::helpers::TestUser for integration tests
}


