//! Downloads and validates an image from Google Cloud Storage.
//!
//! This function combines the download and validation steps for images,
//! ensuring that downloaded content is within acceptable size limits.
//! Provides a convenient interface for the watermarking workflow.

use crate::services::watermarking::watermark_error::WatermarkError;
use crate::services::watermarking::download_asset_bytes::download_asset_bytes;
use crate::services::watermarking::validate_bytes_size::validate_bytes_size;
use crate::services::gcs::gcs_operations::GCSOperations;

/// Downloads and validates an image from GCS
pub async fn download_and_validate_image(
    gcs_client: &std::sync::Arc<dyn GCSOperations>,
    asset_url: &str,
) -> std::result::Result<std::vec::Vec<u8>, WatermarkError> {
    log::info!("Downloading image from: {}", asset_url);
    let image_bytes = download_asset_bytes(gcs_client, asset_url).await?;
    validate_bytes_size(&image_bytes)?;
    log::info!("Image downloaded and validated: {} bytes", image_bytes.len());
    std::result::Result::Ok(image_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_size_validation() {
        // Test that the validation logic is properly applied
        let small_image = vec![0u8; 1024]; // 1KB
        assert!(validate_bytes_size(&small_image).is_ok());
        
        let medium_image = vec![0u8; 10 * 1024 * 1024]; // 10MB  
        assert!(validate_bytes_size(&medium_image).is_ok());
    }

    #[test]
    fn test_empty_image_handling() {
        let empty_image = vec![];
        assert!(validate_bytes_size(&empty_image).is_ok());
    }
}
