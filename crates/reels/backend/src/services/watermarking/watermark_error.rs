//! Error types for watermarking operations.
//!
//! This module defines the comprehensive error handling for the watermarking service.
//! It includes errors for asset operations, image processing, validation, and external service failures.
//! Uses thiserror for automatic error conversions and structured error messages.

use crate::services::watermarking::photon_processor::PhotonError;

/// Error types for watermarking operations
#[derive(Debug, thiserror::Error)]
pub enum WatermarkError {
    #[error("Asset not found: {0}")]
    AssetNotFound(uuid::Uuid),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Image processing error: {0}")]
    ImageProcessing(std::string::String),
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("GCS error: {0}")]
    Gcs(#[from] anyhow::Error),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(std::string::String),
    #[error("Photon processing error: {0}")]
    PhotonProcessing(#[from] PhotonError),
    #[error("File size limit exceeded: {0} bytes (max: {1} bytes)")]
    FileSizeExceeded(u64, u64),
    #[error("Processing timeout exceeded")]
    ProcessingTimeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_error_types() {
        let asset_id = uuid::Uuid::new_v4();
        
        // Test that errors are created correctly and contain expected information for debugging
        let asset_error = WatermarkError::AssetNotFound(asset_id);
        assert!(matches!(asset_error, WatermarkError::AssetNotFound(_)));
        
        let config_error = WatermarkError::InvalidConfig("Test config error".to_string());
        assert!(matches!(config_error, WatermarkError::InvalidConfig(_)));
        
        let file_size_error = WatermarkError::FileSizeExceeded(1000, 500);
        assert!(matches!(file_size_error, WatermarkError::FileSizeExceeded(1000, 500)));
        
        // Test that error messages contain relevant information for debugging
        let error_msg = asset_error.to_string();
        assert!(error_msg.contains(&asset_id.to_string()));
    }

    #[test]
    fn test_watermark_error_from_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let watermark_error = WatermarkError::from(io_error);
        assert!(matches!(watermark_error, WatermarkError::FileSystem(_)));
        
        let photon_error = PhotonError::InvalidConfig("Test photon error".to_string());
        let watermark_error = WatermarkError::from(photon_error);
        assert!(matches!(watermark_error, WatermarkError::PhotonProcessing(_)));
    }
}
