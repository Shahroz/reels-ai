//! Trait defining the core GCS operations needed by application handlers.
//!
//! This allows for dependency injection and testing with different implementations.
//! Generic methods are excluded to maintain object safety for trait objects.

/// Trait defining the core GCS operations needed by application handlers.
/// This allows for dependency injection and testing with different implementations.
/// Note: Generic methods are excluded to maintain object safety for trait objects.

/// URL format for GCS upload operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UrlFormat {
    /// Returns gs://bucket/object format URLs
    GsProtocol,
    /// Returns https://storage.googleapis.com/bucket/object format URLs
    HttpsPublic,
}

impl Default for UrlFormat {
    fn default() -> Self {
        Self::HttpsPublic
    }
}

#[async_trait::async_trait]
pub trait GCSOperations: Send + Sync {
    /// Uploads raw byte data to Google Cloud Storage.
    /// Returns the URL of the uploaded object in the specified format.
    async fn upload_raw_bytes(
        &self,
        bucket_name: &str,
        object_name: &str,
        content_type: &str,
        data: Vec<u8>,
        disable_cache: bool,
        url_format: UrlFormat,
    ) -> anyhow::Result<String>;

    /// Deletes an object from Google Cloud Storage.
    async fn delete_object(&self, bucket_name: &str, object_name: &str) -> anyhow::Result<()>;

    /// Downloads an object from GCS and returns its content as a String.
    async fn download_object_as_string(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> anyhow::Result<std::string::String>;

    /// Downloads an object from GCS and returns its raw bytes.
    async fn download_object_as_bytes(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> anyhow::Result<std::vec::Vec<u8>>;

    /// Enables downcasting to concrete types for diagnostic purposes
    fn as_any(&self) -> &dyn std::any::Any;
} 