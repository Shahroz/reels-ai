//! Error type for asset enhancement operations.
//!
//! Defines `EnhanceAssetError` enum with variants for various failure modes
//! during asset enhancement: invalid IDs, missing assets, type mismatches,
//! enhancement failures, processing errors, and save failures.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from enhance_asset.rs

/// Error type for asset enhancement operations
#[derive(Debug, thiserror::Error)]
pub enum EnhanceAssetError {
    #[error("Invalid asset ID format: {0}")]
    InvalidAssetId(String),
    #[error("Asset not found or access denied")]
    AssetNotFound,
    #[error("Asset must be an image to enhance with AI")]
    NonImageAsset,
    #[error("AI enhancement failed: {0}")]
    EnhancementFailed(String),
    #[error("Failed to process enhancement results: {0}")]
    ProcessingFailed(String),
    #[error("Failed to save enhanced assets: {0}")]
    SaveFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EnhanceAssetError::InvalidAssetId("test-id".to_string());
        assert_eq!(err.to_string(), "Invalid asset ID format: test-id");
        
        let err = EnhanceAssetError::AssetNotFound;
        assert_eq!(err.to_string(), "Asset not found or access denied");
        
        let err = EnhanceAssetError::NonImageAsset;
        assert_eq!(err.to_string(), "Asset must be an image to enhance with AI");
    }

    #[test]
    fn test_error_variants() {
        let err = EnhanceAssetError::EnhancementFailed("network error".to_string());
        assert!(matches!(err, EnhanceAssetError::EnhancementFailed(_)));
        
        let err = EnhanceAssetError::ProcessingFailed("parse error".to_string());
        assert!(matches!(err, EnhanceAssetError::ProcessingFailed(_)));
        
        let err = EnhanceAssetError::SaveFailed("db error".to_string());
        assert!(matches!(err, EnhanceAssetError::SaveFailed(_)));
    }
}


