//! Extended trait for generic GCS operations.
//!
//! This provides the serialization methods for direct use, not trait objects.
//! Provides generic serialization methods that asset handlers don't need.

/// Extended trait for generic operations (for direct use, not trait objects)
/// This provides the serialization methods that asset handlers don't need
#[async_trait::async_trait]
pub trait GCSSerializationOps {
    /// Writes serialized data to Google Cloud Storage at the given path within the specified bucket.
    async fn write_to_gcs<T: serde::Serialize + Send + Sync>(
        &self,
        bucket_name: &str,
        path: &str,
        data: &T,
    ) -> anyhow::Result<()>;

    /// Reads and deserializes data from Google Cloud Storage at the given path within the specified bucket.
    async fn read_from_gcs<T: serde::de::DeserializeOwned + Send + Sync>(
        &self,
        bucket_name: &str,
        path: &str,
    ) -> anyhow::Result<T>;

    /// Lists all objects with a given prefix within the specified bucket.
    async fn list_objects_with_prefix(
        &self,
        bucket_name: &str,
        prefix: &str,
    ) -> anyhow::Result<std::vec::Vec<std::string::String>>;
} 