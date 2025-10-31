//! Mock GCS service implementation for testing.
//!
//! This service provides a fake GCS implementation that returns
//! predictable URLs without performing actual uploads. Used exclusively in
//! test environments to avoid external dependencies and network calls.
//! The returned URLs follow the expected format while maintaining fast
//! and reliable test execution without requiring GCS emulator infrastructure.
//!
//! Revision History:
//! - 2025-01-XX: Created mock GCS service to replace emulator-based testing
//! - Prior revision history not available

/// Mock GCS service that returns dummy URLs for testing
pub struct MockGCSService;

impl MockGCSService {
    /// Create a new mock GCS service
    pub fn new() -> Self {
        Self
    }
}

impl std::default::Default for MockGCSService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl crate::services::gcs::gcs_service::GCSService for MockGCSService {
    async fn upload_raw_bytes(
        &self,
        bucket_name: &str,
        object_name: &str,
        _content_type: &str,
        data: std::vec::Vec<u8>,
        _public: bool,
        url_format: crate::services::gcs::gcs_operations::UrlFormat,
    ) -> std::result::Result<std::string::String, std::string::String> {
        // Simulate upload validation
        if data.is_empty() {
            return std::result::Result::Err(std::string::String::from("Cannot upload empty data"));
        }
        
        // Return predictable test URL based on format
        let base_url = match url_format {
            crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic => {
                std::format!("https://storage.googleapis.com/{}/{}", bucket_name, object_name)
            }
            crate::services::gcs::gcs_operations::UrlFormat::GsProtocol => {
                std::format!("gs://{}/{}", bucket_name, object_name)
            }
        };
        
        std::result::Result::Ok(base_url)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_mock_upload_returns_expected_url_format() {
        let service = super::MockGCSService::new();
        let data = std::vec![1, 2, 3, 4];
        
        let result = <super::MockGCSService as crate::services::gcs::gcs_service::GCSService>::upload_raw_bytes(
            &service,
            "test-bucket",
            "test-object.png",
            "image/png",
            data,
            true,
            crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic,
        ).await;
        
        std::assert!(result.is_ok());
        let url = result.unwrap();
        std::assert_eq!(url, std::string::String::from("https://storage.googleapis.com/test-bucket/test-object.png"));
    }

    #[tokio::test]
    async fn test_mock_upload_handles_empty_data() {
        let service = super::MockGCSService::new();
        let empty_data = std::vec![];
        
        let result = <super::MockGCSService as crate::services::gcs::gcs_service::GCSService>::upload_raw_bytes(
            &service,
            "test-bucket",
            "test-object.png",
            "image/png",
            empty_data,
            true,
            crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic,
        ).await;
        
        std::assert!(result.is_err());
        std::assert_eq!(result.unwrap_err(), std::string::String::from("Cannot upload empty data"));
    }

    #[tokio::test]
    async fn test_mock_upload_gs_uri_format() {
        let service = super::MockGCSService::new();
        let data = std::vec![1, 2, 3, 4];
        
        let result = <super::MockGCSService as crate::services::gcs::gcs_service::GCSService>::upload_raw_bytes(
            &service,
            "test-bucket",
            "styles/123/screenshot.png",
            "image/png",
            data,
            false,
            crate::services::gcs::gcs_operations::UrlFormat::GsProtocol,
        ).await;
        
        std::assert!(result.is_ok());
        let url = result.unwrap();
        std::assert_eq!(url, std::string::String::from("gs://test-bucket/styles/123/screenshot.png"));
    }
}
