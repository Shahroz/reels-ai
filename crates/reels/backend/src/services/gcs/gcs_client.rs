//! Struct for handling Google Cloud Storage operations independently of bucket.
//!
//! The GCSClient provides methods for reading, writing, uploading, and deleting objects
//! from Google Cloud Storage. It includes both raw operations and serialization support.

use crate::services::gcs::gcs_operations::UrlFormat;
use crate::services::gcs::gcs_operations::GCSOperations;
use crate::services::gcs::gcs_serialization_ops::GCSSerializationOps;
use anyhow::Context;

/// Struct for handling Google Cloud Storage operations independently of bucket
#[derive(Clone)]
pub struct GCSClient {
    pub client: std::sync::Arc<tokio::sync::RwLock<std::option::Option<google_cloud_storage::client::Client>>>,
}

impl Default for GCSClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GCSClient {
    /// Creates a new instance of GCSClient
    pub fn new() -> Self {
        let client = std::sync::Arc::new(tokio::sync::RwLock::new(std::option::Option::None));
        Self { client }
    }

    /// Writes data to Google Cloud Storage at the given path within the specified bucket
    #[tracing::instrument(skip(self, data))]
    pub async fn write_to_gcs<T: serde::Serialize>(
        &self,
        bucket_name: &str,
        path: &str,
        data: &T,
    ) -> anyhow::Result<()> {
        let client = self.get_client().await?;

        let upload_type = google_cloud_storage::http::objects::upload::UploadType::Simple(
            google_cloud_storage::http::objects::upload::Media::new(path.to_string())
        );
        let serialized_data = serde_json::to_string(data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize data: {}", e))?;
        let bytes = serialized_data.into_bytes();

        let result = client
            .upload_object(
                &google_cloud_storage::http::objects::upload::UploadObjectRequest {
                    bucket: bucket_name.to_string(),
                    ..Default::default()
                },
                bytes,
                &upload_type,
            )
            .await;

        result.map_err(|e| anyhow::anyhow!("Failed to upload to GCS: {:?}", e))?;

        std::result::Result::Ok(())
    }

    /// Reads data from Google Cloud Storage at the given path within the specified bucket
    #[tracing::instrument(skip(self))]
    pub async fn read_from_gcs<T: serde::de::DeserializeOwned>(
        &self,
        bucket_name: &str,
        path: &str,
    ) -> anyhow::Result<T> {
        let client = self.get_client().await?;

        let bytes = client
            .download_object(
                &google_cloud_storage::http::objects::get::GetObjectRequest {
                    bucket: bucket_name.to_string(),
                    object: path.to_string(),
                    ..Default::default()
                },
                &google_cloud_storage::http::objects::download::Range::default(),
            )
            .await
            .context("Failed to download object")?;

        let data: T = serde_json::from_slice(&bytes)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize data: {}", e))?;

        std::result::Result::Ok(data)
    }

    /// Lists all objects with a given prefix within the specified bucket
    #[tracing::instrument(skip(self))]
    pub async fn list_objects_with_prefix(
        &self,
        bucket_name: &str,
        prefix: &str,
    ) -> anyhow::Result<std::vec::Vec<std::string::String>> {
        let client = self.get_client().await?;

        let list_request = google_cloud_storage::http::objects::list::ListObjectsRequest {
            bucket: bucket_name.to_string(),
            prefix: std::option::Option::Some(prefix.to_string()),
            ..Default::default()
        };

        let objects = client
            .list_objects(&list_request)
            .await
            .context("Failed to list objects")?
            .items
            .context("No items found")?;

        std::result::Result::Ok(objects.into_iter().map(|obj| obj.name).collect())
    }

    /// Uploads raw byte data to Google Cloud Storage.
    /// Returns the public URL of the uploaded object.
    #[tracing::instrument(skip(self, data))]
    pub async fn upload_raw_bytes(
        &self,
        bucket_name: &str,
        object_name: &str,
        content_type: &str,
        data: std::vec::Vec<u8>,
        disable_cache: bool,
        url_format: UrlFormat,
    ) -> anyhow::Result<std::string::String> {
        let client = self.get_client().await?;

        let upload_request = google_cloud_storage::http::objects::upload::UploadObjectRequest {
            bucket: bucket_name.to_string(),
            ..Default::default()
        };

        let upload_type = if disable_cache {
            let object_metadata = google_cloud_storage::http::objects::Object {
                name: object_name.to_string(),
                content_type: Some(content_type.to_string()),
                cache_control: Some("no-cache, no-store, must-revalidate".to_string()),
                ..Default::default()
            };
            google_cloud_storage::http::objects::upload::UploadType::Multipart(Box::new(
                object_metadata,
            ))
        } else {
            let mut media =
                google_cloud_storage::http::objects::upload::Media::new(object_name.to_string());
            media.content_type = std::borrow::Cow::Owned(content_type.to_string());
            google_cloud_storage::http::objects::upload::UploadType::Simple(media)
        };

        client
            .upload_object(&upload_request, data, &upload_type)
            .await
            .with_context(|| {
                format!(
                    "Failed to upload raw bytes to GCS. Bucket: '{bucket_name}', Object: '{object_name}', Content-Type: '{content_type}', Disable-Cache: {disable_cache}"
                )
            })?;

        // Construct the GCS URL - use emulator URL only if emulator is actually configured with non-empty host
        let base_url = if let Ok(emulator_host) = std::env::var("STORAGE_EMULATOR_HOST") {
            if !emulator_host.trim().is_empty() {
                // For emulator, use the emulator endpoint format
                format!("{emulator_host}/storage/v1/b/{bucket_name}/o/{object_name}")
            } else {
                // For production (empty emulator host), use the requested URL format
                match url_format {
                    UrlFormat::GsProtocol => format!("gs://{bucket_name}/{object_name}"),
                    UrlFormat::HttpsPublic => format!("https://storage.googleapis.com/{bucket_name}/{object_name}"),
                }
            }
        } else {
            // For production, use the requested URL format
            match url_format {
                UrlFormat::GsProtocol => format!("gs://{bucket_name}/{object_name}"),
                UrlFormat::HttpsPublic => format!("https://storage.googleapis.com/{bucket_name}/{object_name}"),
            }
        };
        Ok(base_url)
    }


    /// Deletes an object from Google Cloud Storage.
    #[tracing::instrument(skip(self))]
    pub async fn delete_object(&self, bucket_name: &str, object_name: &str) -> anyhow::Result<()> {
        let client = self.get_client().await?;

        let request = google_cloud_storage::http::objects::delete::DeleteObjectRequest {
            bucket: bucket_name.to_string(),
            object: object_name.to_string(),
            ..Default::default()
        };

        client.delete_object(&request).await.context(std::format!(
            "Failed to delete object '{object_name}' from bucket '{bucket_name}'"
        ))?;

        std::result::Result::Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_client(&self) -> anyhow::Result<google_cloud_storage::client::Client> {
        // First check if client already exists with read lock
        {
            let client_read = self.client.read().await;
            if let Some(ref client) = *client_read {
                return std::result::Result::Ok(client.clone());
            }
        }

        // Client doesn't exist, need to initialize with write lock
        let mut client_write = self.client.write().await;
        
        // Double-check after acquiring write lock to avoid race condition
        if let Some(ref client) = *client_write {
            return std::result::Result::Ok(client.clone());
        }

        // Initialize client config (this is the potentially blocking part)
        let client_config = if let std::result::Result::Ok(emulator_host) = std::env::var("STORAGE_EMULATOR_HOST") {
            if !emulator_host.trim().is_empty() {
                log::info!("Using GCS emulator at: {emulator_host}");
                google_cloud_storage::client::ClientConfig {
                    storage_endpoint: emulator_host,
                    ..Default::default()
                }.anonymous()
            } else {
                log::warn!("STORAGE_EMULATOR_HOST is set but empty, falling back to production GCS");
                // Drop write lock before async auth call to prevent deadlock
                drop(client_write);
                let config = google_cloud_storage::client::ClientConfig::default()
                    .with_auth()
                    .await
                    .with_context(|| "Failed to create GCS client config with authentication. Check your GCP credentials and permissions.")?;
                
                // Re-acquire write lock after auth
                client_write = self.client.write().await;
                // Triple-check after re-acquiring lock
                if let Some(ref client) = *client_write {
                    return std::result::Result::Ok(client.clone());
                }
                config
            }
        } else {
            log::info!("Using production GCS configuration with authentication");
            // Drop write lock before async auth call to prevent deadlock
            drop(client_write);
            let config = google_cloud_storage::client::ClientConfig::default()
                .with_auth()
                .await
                .with_context(|| "Failed to create GCS client config with authentication. Check your GCP credentials and permissions.")?;
            
            // Re-acquire write lock after auth
            client_write = self.client.write().await;
            // Triple-check after re-acquiring lock
            if let Some(ref client) = *client_write {
                return std::result::Result::Ok(client.clone());
            }
            config
        };

        let client = google_cloud_storage::client::Client::new(client_config);
        *client_write = std::option::Option::Some(client.clone());
        log::info!("GCS client initialized successfully");
        
        std::result::Result::Ok(client)
    }

    /// Downloads an object from GCS and returns its content as a String.
    #[tracing::instrument(skip(self))]
    pub async fn download_object_as_string(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> anyhow::Result<std::string::String> {
        let client = self.get_client().await?;

        let bytes = client
            .download_object(
                &google_cloud_storage::http::objects::get::GetObjectRequest {
                    bucket: bucket_name.to_string(),
                    object: object_name.to_string(),
                    ..Default::default()
                },
                &google_cloud_storage::http::objects::download::Range::default(),
            )
            .await
            .with_context(|| {
                std::format!(
                    "Failed to download object '{object_name}' from bucket '{bucket_name}'"
                )
            })?;

        std::string::String::from_utf8(bytes).with_context(|| {
            std::format!(
                "Failed to convert downloaded bytes to UTF-8 string for object '{object_name}'"
            )
        })
    }

    /// Downloads an object from GCS and returns its raw bytes.
    #[tracing::instrument(skip(self))]
    pub async fn download_object_as_bytes(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> anyhow::Result<std::vec::Vec<u8>> {
        let client = self.get_client().await?;

        let bytes = client
            .download_object(
                &google_cloud_storage::http::objects::get::GetObjectRequest {
                    bucket: bucket_name.to_string(),
                    object: object_name.to_string(),
                    ..Default::default()
                },
                &google_cloud_storage::http::objects::download::Range::default(),
            )
            .await
            .with_context(|| {
                std::format!(
                    "Failed to download object '{object_name}' from bucket '{bucket_name}'"
                )
            })?;

        std::result::Result::Ok(bytes)
    }

    /// Generates a signed URL for uploading to Google Cloud Storage.
    /// Returns a time-limited URL that allows direct uploads to GCS.
    #[tracing::instrument(skip(self))]
    pub async fn generate_signed_upload_url(
        &self,
        bucket_name: &str,
        object_name: &str,
        content_type: std::option::Option<std::string::String>,
        expires_in: std::time::Duration,
    ) -> anyhow::Result<std::string::String> {
        let client = self.get_client().await?;

        let options = google_cloud_storage::sign::SignedURLOptions {
            method: google_cloud_storage::sign::SignedURLMethod::PUT,
            expires: expires_in,
            content_type,
            ..Default::default()
        };

        let signed_url = client
            .signed_url(bucket_name, object_name, std::option::Option::None, std::option::Option::None, options)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to generate signed URL: {}", e))?;

        std::result::Result::Ok(signed_url)
    }

    /// Verifies that an object exists in Google Cloud Storage.
    /// Returns true if the object exists, false otherwise.
    #[tracing::instrument(skip(self))]
    pub async fn object_exists(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> anyhow::Result<bool> {
        let client = self.get_client().await?;

        let get_request = google_cloud_storage::http::objects::get::GetObjectRequest {
            bucket: bucket_name.to_string(),
            object: object_name.to_string(),
            ..Default::default()
        };

        match client.get_object(&get_request).await {
            std::result::Result::Ok(_) => std::result::Result::Ok(true),
            std::result::Result::Err(google_cloud_storage::http::Error::HttpClient(ref e)) if e.status().map(|s| s.as_u16()) == std::option::Option::Some(404) => std::result::Result::Ok(false),
            std::result::Result::Err(e) => std::result::Result::Err(anyhow::anyhow!("Failed to verify object existence: {}", e)),
        }
    }
}

/// Implementation of GCSOperations trait for the production GCSClient
#[async_trait::async_trait]
impl GCSOperations for GCSClient {
    async fn upload_raw_bytes(
        &self,
        bucket_name: &str,
        object_name: &str,
        content_type: &str,
        data: Vec<u8>,
        disable_cache: bool,
        url_format: UrlFormat,
    ) -> anyhow::Result<String> {
        self.upload_raw_bytes(bucket_name, object_name, content_type, data, disable_cache, url_format)
            .await
    }

    async fn delete_object(&self, bucket_name: &str, object_name: &str) -> anyhow::Result<()> {
        self.delete_object(bucket_name, object_name).await
    }

    async fn download_object_as_string(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> anyhow::Result<std::string::String> {
        self.download_object_as_string(bucket_name, object_name).await
    }

    async fn download_object_as_bytes(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> anyhow::Result<std::vec::Vec<u8>> {
        self.download_object_as_bytes(bucket_name, object_name).await
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Implementation of GCSSerializationOps trait for the production GCSClient
#[async_trait::async_trait]
impl GCSSerializationOps for GCSClient {
    async fn write_to_gcs<T: serde::Serialize + Send + Sync>(
        &self,
        bucket_name: &str,
        path: &str,
        data: &T,
    ) -> anyhow::Result<()> {
        self.write_to_gcs(bucket_name, path, data).await
    }

    async fn read_from_gcs<T: serde::de::DeserializeOwned + Send + Sync>(
        &self,
        bucket_name: &str,
        path: &str,
    ) -> anyhow::Result<T> {
        self.read_from_gcs(bucket_name, path).await
    }

    async fn list_objects_with_prefix(
        &self,
        bucket_name: &str,
        prefix: &str,
    ) -> anyhow::Result<std::vec::Vec<std::string::String>> {
        self.list_objects_with_prefix(bucket_name, prefix).await
    }
} 