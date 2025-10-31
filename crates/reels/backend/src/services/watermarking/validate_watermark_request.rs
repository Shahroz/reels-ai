//! Validates watermark request parameters.
//!
//! This function ensures that watermark requests are within acceptable limits
//! and contain valid data before processing begins.
//! Prevents abuse by limiting the number of watermarks per request.

use crate::schemas::watermark_schemas::WatermarkDefinition;
use crate::services::watermarking::watermark_error::WatermarkError;

/// Validates the watermark request parameters
pub fn validate_watermark_request(watermarks: &[WatermarkDefinition]) -> std::result::Result<(), WatermarkError> {
    if watermarks.is_empty() {
        return std::result::Result::Err(WatermarkError::InvalidConfig(
            std::string::String::from("No watermarks provided")
        ));
    }
    
    // Validate watermark limit to prevent abuse
    const MAX_WATERMARKS: usize = 10;
    if watermarks.len() > MAX_WATERMARKS {
        return std::result::Result::Err(WatermarkError::InvalidConfig(
            std::format!("Too many watermarks requested: {} (maximum: {})", 
                watermarks.len(), MAX_WATERMARKS)
        ));
    }
    
    std::result::Result::Ok(())
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
    fn test_validate_watermark_request_empty() {
        let empty_watermarks = vec![];
        let result = validate_watermark_request(&empty_watermarks);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No watermarks provided"));
    }

    #[test]
    fn test_validate_watermark_request_too_many() {
        let watermarks: std::vec::Vec<WatermarkDefinition> = (0..15)
            .map(|_| create_test_watermark(uuid::Uuid::new_v4()))
            .collect();
        
        let result = validate_watermark_request(&watermarks);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Too many watermarks requested"));
    }

    #[test]
    fn test_validate_watermark_request_valid() {
        let watermarks = vec![
            create_test_watermark(uuid::Uuid::new_v4()),
            create_test_watermark(uuid::Uuid::new_v4()),
        ];
        
        let result = validate_watermark_request(&watermarks);
        assert!(result.is_ok());
    }
}
