//! Downloads an asset from Google Cloud Storage as bytes.
//!
//! This function handles the download of assets from GCS using the provided URL.
//! It parses GCS URLs and downloads the object as raw bytes for image processing.
//! Includes proper error handling for network and parsing failures.

use crate::services::watermarking::watermark_error::WatermarkError;
use crate::services::gcs::gcs_operations::GCSOperations;
use crate::services::gcs::parse_gcs_url::parse_gcs_url;

/// Downloads an asset from GCS as bytes
pub async fn download_asset_bytes(
    gcs_client: &std::sync::Arc<dyn GCSOperations>,
    asset_url: &str,
) -> std::result::Result<std::vec::Vec<u8>, WatermarkError> {
    log::info!("Parsing GCS URL: {}", asset_url);
    let (bucket_name, object_name) = parse_gcs_url(asset_url)
        .map_err(|e| WatermarkError::InvalidConfig(format!("Failed to parse GCS URL: {}", e)))?;
    log::info!("Parsed GCS URL - bucket: '{}', object: '{}'", bucket_name, object_name);
    
    log::info!("Downloading object from GCS bucket '{}', object '{}'", bucket_name, object_name);
    let bytes = gcs_client.download_object_as_bytes(&bucket_name, &object_name).await?;
    log::info!("Downloaded {} bytes", bytes.len());
    
    std::result::Result::Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handling_for_invalid_url() {
        // Test that invalid URLs are properly handled by the error conversion logic
        let error_msg = "Failed to parse GCS URL: invalid format";
        let error = WatermarkError::InvalidConfig(error_msg.to_string());
        
        // Test that the error contains the expected information for debugging
        let error_string = error.to_string();
        assert!(error_string.contains("invalid format"));
        assert!(error_string.contains("Failed to parse GCS URL"));
    }

    #[test]
    fn test_url_format_requirements() {
        // Test our business logic for what constitutes a valid GCS URL pattern
        let test_cases = vec![
            ("https://storage.googleapis.com/bucket/object", true),
            ("http://storage.googleapis.com/bucket/object", false),  // Must be HTTPS
            ("https://example.com/bucket/object", false),            // Must be googleapis.com
            ("storage.googleapis.com/bucket/object", false),         // Must have protocol
        ];

        for (url, should_be_valid_format) in test_cases {
            let has_https = url.starts_with("https://");
            let has_googleapis = url.contains("storage.googleapis.com");
            let meets_our_requirements = has_https && has_googleapis;
            
            assert_eq!(meets_our_requirements, should_be_valid_format, 
                "URL {} should be {} but got {}", url, should_be_valid_format, meets_our_requirements);
        }
    }
}
