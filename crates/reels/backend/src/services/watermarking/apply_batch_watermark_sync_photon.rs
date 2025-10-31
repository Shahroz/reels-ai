//! Applies multiple watermarks to an asset using photon-rs batch processing.
//!
//! This function orchestrates the complete watermarking workflow including validation,
//! asset fetching, image processing, and result creation. It processes watermarks
//! sequentially to maintain quality and handles all error scenarios properly.
//! The main entry point for the watermarking service.

use crate::schemas::watermark_schemas::{WatermarkResponse, WatermarkDefinition};
use crate::services::watermarking::watermark_error::WatermarkError;
use crate::services::watermarking::validate_watermark_request::validate_watermark_request;
use crate::services::watermarking::fetch_and_validate_source_asset::fetch_and_validate_source_asset;
use crate::services::watermarking::download_and_validate_image::download_and_validate_image;
use crate::services::watermarking::process_watermarks::process_watermarks;
use crate::services::watermarking::create_final_watermarked_asset::create_final_watermarked_asset;
use crate::services::gcs::gcs_operations::GCSOperations;

/// Applies multiple watermarks to an asset using photon-rs (batch processing)
pub async fn apply_batch_watermark_sync_photon(
    pool: &sqlx::PgPool,
    gcs_client: &std::sync::Arc<dyn GCSOperations>,
    user_id: uuid::Uuid,
    source_asset_id: uuid::Uuid,
    watermarks: std::vec::Vec<WatermarkDefinition>,
) -> std::result::Result<WatermarkResponse, WatermarkError> {
    let start_time = std::time::Instant::now();
    
    log::info!("Starting photon-rs batch watermark application - user: {}, source: {}, watermarks: {}", 
               user_id, source_asset_id, watermarks.len());
    
    // Validate input parameters
    validate_watermark_request(&watermarks)?;
    
    // Fetch and validate source asset
    let source_asset = fetch_and_validate_source_asset(pool, source_asset_id, user_id).await?;
    
    // Download and validate source image
    let mut current_image_bytes = download_and_validate_image(gcs_client, &source_asset.url).await?;
    
    // Process each watermark sequentially
    current_image_bytes = process_watermarks(
        pool, 
        gcs_client, 
        user_id, 
        &source_asset, 
        &watermarks, 
        current_image_bytes
    ).await?;
    
    // Create and upload final result
    let watermarked_asset = create_final_watermarked_asset(
        pool,
        gcs_client,
        user_id,
        &source_asset,
        current_image_bytes,
    ).await?;
    
    let processing_time = start_time.elapsed();
    let processing_time_ms = processing_time.as_millis() as i64;
    
    log::info!("Photon-rs batch watermarking completed successfully - created single asset: {}", watermarked_asset.id);
    
    std::result::Result::Ok(WatermarkResponse {
        result_asset_id: watermarked_asset.id,
        result_asset_url: watermarked_asset.url,
        processing_time_ms,
        completed_at: chrono::Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_response_construction() {
        let asset_id = uuid::Uuid::new_v4();
        let response = WatermarkResponse {
            result_asset_id: asset_id,
            result_asset_url: "https://example.com/watermarked.png".to_string(),
            processing_time_ms: 1500,
            completed_at: chrono::Utc::now(),
        };
        
        assert_eq!(response.result_asset_id, asset_id);
        assert_eq!(response.result_asset_url, "https://example.com/watermarked.png");
        assert_eq!(response.processing_time_ms, 1500);
    }

    #[test]
    fn test_processing_time_calculation() {
        let start = std::time::Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_millis() as i64;
        
        assert!(elapsed_ms >= 10); // Should be at least 10ms
        assert!(elapsed_ms < 1000); // Should be less than 1 second for this test
    }
}
