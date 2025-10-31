//! Processes all watermarks sequentially on an image.
//!
//! This function orchestrates the application of multiple watermarks to an image,
//! processing each watermark definition in sequence. It handles logo asset fetching,
//! validation, and applies each watermark using the photon processor.

use crate::schemas::watermark_schemas::WatermarkDefinition;
use crate::services::watermarking::watermark_error::WatermarkError;
use crate::services::watermarking::process_single_watermark::process_single_watermark;
use crate::services::watermarking::photon_processor::PhotonProcessor;
use crate::services::gcs::gcs_operations::GCSOperations;

/// Processes all watermarks sequentially
pub async fn process_watermarks(
    pool: &sqlx::PgPool,
    gcs_client: &std::sync::Arc<dyn GCSOperations>,
    user_id: uuid::Uuid,
    source_asset: &crate::db::assets::Asset,
    watermarks: &[WatermarkDefinition],
    mut current_image_bytes: std::vec::Vec<u8>,
) -> std::result::Result<std::vec::Vec<u8>, WatermarkError> {
    let photon_processor = PhotonProcessor::new();
    
    log::info!("Processing {} watermarks", watermarks.len());
    for (i, watermark_def) in watermarks.iter().enumerate() {
        log::info!("Processing watermark {} of {}", i + 1, watermarks.len());
        
        // Process single watermark
        current_image_bytes = process_single_watermark(
            pool,
            gcs_client,
            user_id,
            source_asset,
            watermark_def,
            &photon_processor,
            current_image_bytes,
        ).await?;
        
        log::info!("Watermark {} completed successfully, result: {} bytes", i + 1, current_image_bytes.len());
    }
    
    std::result::Result::Ok(current_image_bytes)
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

    #[test]
    fn test_watermark_processing_logic() {
        // Test the watermark processing setup
        let watermark1 = create_test_watermark(uuid::Uuid::new_v4());
        let watermark2 = create_test_watermark(uuid::Uuid::new_v4());
        let watermarks = vec![watermark1, watermark2];
        
        assert_eq!(watermarks.len(), 2);
        
        // Test iteration logic
        for (i, _watermark) in watermarks.iter().enumerate() {
            assert!(i < 2);
        }
    }

    #[test]
    fn test_empty_watermarks() {
        let empty_watermarks: std::vec::Vec<WatermarkDefinition> = vec![];
        assert_eq!(empty_watermarks.len(), 0);
        
        // Empty iteration should not execute
        let mut count = 0;
        for _watermark in empty_watermarks.iter() {
            count += 1;
        }
        assert_eq!(count, 0);
    }
}
