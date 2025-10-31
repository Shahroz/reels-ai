//! Trait for Google Cloud Storage operations.
//!
//! This trait provides a common interface for GCS upload functionality,
//! enabling dependency injection and testability in the application.
//! Implementations can vary from production services using real GCS APIs
//! to mock services returning dummy data for testing scenarios.
//! The trait is async and thread-safe for use in concurrent environments.
//!
//! Revision History:
//! - 2025-01-XX: Created GCS service trait for dependency injection refactor
//! - Prior revision history not available

/// Trait for Google Cloud Storage operations
#[async_trait::async_trait]
pub trait GCSService: Send + Sync {
    /// Upload raw bytes to GCS bucket
    /// 
    /// # Arguments
    /// * `bucket_name` - The GCS bucket name
    /// * `object_name` - The object path within the bucket
    /// * `content_type` - MIME type of the content
    /// * `data` - Raw bytes to upload
    /// * `public` - Whether to make the object publicly accessible
    /// * `url_format` - Format for the returned URL
    /// 
    /// # Returns
    /// GCS URL on success, error message on failure
    async fn upload_raw_bytes(
        &self,
        bucket_name: &str,
        object_name: &str,
        content_type: &str,
        data: std::vec::Vec<u8>,
        public: bool,
        url_format: crate::services::gcs::gcs_operations::UrlFormat,
    ) -> std::result::Result<std::string::String, std::string::String>;
}
