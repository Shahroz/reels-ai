//! Validates that assets are suitable for watermarking operations.
//!
//! This function checks that both source and logo assets are image types
//! and compatible for watermarking operations.
//! Prevents processing errors by validating MIME types upfront.

use crate::services::watermarking::watermark_error::WatermarkError;

/// Validates that assets are suitable for watermarking
pub fn validate_image_assets(
    source_asset: &crate::db::assets::Asset,
    logo_asset: &crate::db::assets::Asset,
) -> std::result::Result<(), WatermarkError> {
    // Check if source asset is an image (check for MIME type starting with "image/")
    if !source_asset.r#type.starts_with("image/") {
        return std::result::Result::Err(WatermarkError::InvalidConfig(
            std::format!("Source asset must be an image, got: {}", source_asset.r#type)
        ));
    }
    
    // Check if logo asset is an image (check for MIME type starting with "image/")
    if !logo_asset.r#type.starts_with("image/") {
        return std::result::Result::Err(WatermarkError::InvalidConfig(
            std::format!("Logo asset must be an image, got: {}", logo_asset.r#type)
        ));
    }
    
    log::info!("Asset validation passed - source: {}, logo: {}", source_asset.r#type, logo_asset.r#type);
    std::result::Result::Ok(())
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
    fn test_validate_image_assets_valid() {
        let source_asset = create_test_asset(uuid::Uuid::new_v4(), "source.jpg", "image/jpeg");
        let logo_asset = create_test_asset(uuid::Uuid::new_v4(), "logo.png", "image/png");
        
        let result = validate_image_assets(&source_asset, &logo_asset);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_image_assets_invalid_source() {
        let source_asset = create_test_asset(uuid::Uuid::new_v4(), "document.pdf", "application/pdf");
        let logo_asset = create_test_asset(uuid::Uuid::new_v4(), "logo.png", "image/png");
        
        let result = validate_image_assets(&source_asset, &logo_asset);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Source asset must be an image"));
    }

    #[test]
    fn test_validate_image_assets_invalid_logo() {
        let source_asset = create_test_asset(uuid::Uuid::new_v4(), "source.jpg", "image/jpeg");
        let logo_asset = create_test_asset(uuid::Uuid::new_v4(), "document.txt", "text/plain");
        
        let result = validate_image_assets(&source_asset, &logo_asset);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Logo asset must be an image"));
    }
}
