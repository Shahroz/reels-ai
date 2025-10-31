//! Fetches and validates the source asset for watermarking.
//!
//! This function retrieves a source asset from the database and validates that it belongs
//! to the requesting user and is a valid image type for watermarking operations.
//! Provides proper error handling for missing assets and authorization failures.

use crate::services::watermarking::watermark_error::WatermarkError;
use crate::services::watermarking::get_asset_by_id::get_asset_by_id;

/// Fetches and validates the source asset
pub async fn fetch_and_validate_source_asset(
    pool: &sqlx::PgPool,
    source_asset_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> std::result::Result<crate::db::assets::Asset, WatermarkError> {
    log::info!("Fetching source asset: {}", source_asset_id);
    let source_asset = get_asset_by_id(pool, source_asset_id, user_id).await.map_err(|e| {
        log::error!("Failed to fetch source asset {}: {}", source_asset_id, e);
        e
    })?;
    
    // Validate source asset is an image
    if !source_asset.r#type.starts_with("image/") {
        return std::result::Result::Err(WatermarkError::InvalidConfig(
            std::format!("Source asset must be an image, got: {}", source_asset.r#type)
        ));
    }
    
    log::info!("Source asset fetched: {} ({})", source_asset.name, source_asset.url);
    std::result::Result::Ok(source_asset)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a test asset for testing purposes
    fn create_test_asset(id: uuid::Uuid, name: &str, asset_type: &str) -> crate::db::assets::Asset {
        crate::db::assets::Asset {
            id,
            user_id: std::option::Option::Some(uuid::Uuid::new_v4()),
            name: name.to_string(),
            r#type: asset_type.to_string(),
            gcs_object_name: std::format!("test/{}", name),
            url: std::format!("https://storage.googleapis.com/test-bucket/test/{}", name),
            collection_id: std::option::Option::None,
            metadata: std::option::Option::None,
            created_at: std::option::Option::Some(chrono::Utc::now()),
            updated_at: std::option::Option::Some(chrono::Utc::now()),
            is_public: false,
        }
    }

    #[test]
    fn test_validate_image_asset_types() {
        let source_asset = create_test_asset(uuid::Uuid::new_v4(), "source.jpg", "image/jpeg");
        assert!(source_asset.r#type.starts_with("image/"));
        
        let non_image_asset = create_test_asset(uuid::Uuid::new_v4(), "document.pdf", "application/pdf");
        assert!(!non_image_asset.r#type.starts_with("image/"));
    }
}
