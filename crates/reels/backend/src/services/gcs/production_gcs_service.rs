//! Production GCS service implementation using real Google Cloud Storage.
//!
//! This service provides real GCS upload functionality by wrapping
//! the existing GCS client operations. It handles authentication and
//! error transformation to maintain consistency with the GCS service
//! interface. Used in production environments where actual uploads are
//! required for file storage and serving workflows.
//!
//! Revision History:
//! - 2025-01-XX: Created production GCS service wrapper for dependency injection
//! - Prior revision history not available

/// Production GCS service that uses real Google Cloud Storage
pub struct ProductionGCSService {
    client: std::sync::Arc<crate::services::gcs::gcs_client::GCSClient>,
}

impl ProductionGCSService {
    /// Create a new production GCS service with the given client
    pub fn new(client: std::sync::Arc<crate::services::gcs::gcs_client::GCSClient>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl crate::services::gcs::gcs_service::GCSService for ProductionGCSService {
    async fn upload_raw_bytes(
        &self,
        bucket_name: &str,
        object_name: &str,
        content_type: &str,
        data: std::vec::Vec<u8>,
        public: bool,
        url_format: crate::services::gcs::gcs_operations::UrlFormat,
    ) -> std::result::Result<std::string::String, std::string::String> {
        self.client
            .upload_raw_bytes(bucket_name, object_name, content_type, data, public, url_format)
            .await
            .map_err(|e| std::format!("GCS upload failed for bucket '{}', object '{}' ({}): {e}", bucket_name, object_name, content_type))
    }
}
