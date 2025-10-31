//! Processes a single watermark application.
//!
//! This function handles the complete workflow for applying one watermark to an image.
//! It fetches the logo asset, validates compatibility, downloads the logo, and applies
//! the watermark using the photon processor with the specified configuration.

use crate::schemas::watermark_schemas::WatermarkDefinition;
use crate::services::watermarking::watermark_error::WatermarkError;
use crate::services::watermarking::get_asset_by_id::get_asset_by_id;
use crate::services::watermarking::validate_image_assets::validate_image_assets;
use crate::services::watermarking::download_and_validate_image::download_and_validate_image;
use crate::services::watermarking::photon_processor::PhotonProcessor;
use crate::services::gcs::gcs_operations::GCSOperations;

/// Processes a single watermark application
pub async fn process_single_watermark(
    pool: &sqlx::PgPool,
    gcs_client: &std::sync::Arc<dyn GCSOperations>,
    user_id: uuid::Uuid,
    source_asset: &crate::db::assets::Asset,
    watermark_def: &WatermarkDefinition,
    photon_processor: &PhotonProcessor,
    current_image_bytes: std::vec::Vec<u8>,
) -> std::result::Result<std::vec::Vec<u8>, WatermarkError> {
    // Fetch and validate logo asset
    let logo_asset = get_asset_by_id(pool, watermark_def.logo_asset_id, user_id).await.map_err(|e| {
        log::error!("Failed to fetch logo asset {}: {}", watermark_def.logo_asset_id, e);
        e
    })?;
    log::info!("Logo asset fetched: {} ({})", logo_asset.name, logo_asset.url);
    
    // Validate assets compatibility
    validate_image_assets(source_asset, &logo_asset)?;
    log::info!("Asset validation passed - source: {}, logo: {}", source_asset.r#type, logo_asset.r#type);
    
    // Download and validate logo image
    let logo_bytes = download_and_validate_image(gcs_client, &logo_asset.url).await?;
    
    // Apply watermark using photon-rs
    log::info!("Applying watermark using photon-rs");
    let result_bytes = photon_processor.apply_watermark_from_bytes(
        &current_image_bytes,
        &logo_bytes,
        &watermark_def.config,
    ).await?;
    
    std::result::Result::Ok(result_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::watermark_schemas::{WatermarkConfig, WatermarkPosition, WatermarkSize, CornerPosition};

    /// Creates a test watermark definition
    fn create_test_watermark(logo_asset_id: uuid::Uuid) -> WatermarkDefinition {
        WatermarkDefinition {
            logo_asset_id,
            config: WatermarkConfig {
                position: WatermarkPosition::Corner(CornerPosition::BottomRight),
                size: WatermarkSize::Percentage(15.0),
                opacity: 0.8,
            },
        }
    }

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
    fn test_watermark_definition_validation() {
        let watermark = create_test_watermark(uuid::Uuid::new_v4());
        assert_eq!(watermark.config.opacity, 0.8);
        assert!(matches!(watermark.config.position, WatermarkPosition::Corner(CornerPosition::BottomRight)));
        assert!(matches!(watermark.config.size, WatermarkSize::Percentage(15.0)));
    }

    #[test]
    fn test_asset_validation_logic() {
        let source_asset = create_test_asset(uuid::Uuid::new_v4(), "source.jpg", "image/jpeg");
        let logo_asset = create_test_asset(uuid::Uuid::new_v4(), "logo.png", "image/png");
        
        // Both are images, so validation should conceptually pass
        assert!(source_asset.r#type.starts_with("image/"));
        assert!(logo_asset.r#type.starts_with("image/"));
        
        let result = validate_image_assets(&source_asset, &logo_asset);
        assert!(result.is_ok());
    }
}
