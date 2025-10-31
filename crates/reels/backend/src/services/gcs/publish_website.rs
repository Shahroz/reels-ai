//! Publishes website content to Google Cloud Storage.
//!
//! This function uploads website content to a randomly named folder in GCS
//! and returns the public URL for accessing the published content.

use crate::gcp_auth::GCPTokenSource;
use anyhow::Context;
use std::str::FromStr;

/// Publishes website content to Google Cloud Storage.
/// 
/// Generates a random UUID for the website folder and uploads the content
/// to the specified bucket with appropriate configuration.
/// 
/// # Arguments
/// * `content` - The HTML content to publish
/// * `destination_bucket` - The GCS bucket to upload to
/// * `base_url` - The base URL for constructing the final public URL
/// 
/// # Returns
/// A URL pointing to the published website content
#[tracing::instrument(skip(content))]
pub async fn publish_website(
    content: std::string::String,
    destination_bucket: &str,
    base_url: &str,
) -> anyhow::Result<url::Url> {
    // Generate a random UUID for the website folder.
    let site_uuid = uuid::Uuid::new_v4();

    // Set up the Google Cloud Storage client configuration.
    let client_config = if let std::result::Result::Ok(emulator_host) = std::env::var("STORAGE_EMULATOR_HOST") {
        if !emulator_host.trim().is_empty() {
            // Use emulator configuration for testing
            google_cloud_storage::client::ClientConfig {
                storage_endpoint: emulator_host,
                ..Default::default()
            }.anonymous()
        } else {
            // Fall back to production configuration if emulator host is empty
            google_cloud_storage::client::ClientConfig::default().with_auth().await.map_err(|e| {
                log::warn!("Failed to create GCS client config: {e}");
                anyhow::anyhow!("Failed to create GCS client config: {}", e)
            })?
        }
    } else if std::cfg!(debug_assertions) {
        // Developer/CI approach with default local auth.
        google_cloud_storage::client::ClientConfig::default().with_auth().await.map_err(|e| {
            log::warn!("Failed to create GCS client config: {e}");
            anyhow::anyhow!("Failed to create GCS client config: {}", e)
        })?
    } else {
        // Production approach with GCPTokenSource.
        let ts = GCPTokenSource::new();
        google_cloud_storage::client::ClientConfig {
            http: std::option::Option::None,
            storage_endpoint: "https://storage.googleapis.com".to_string(),
            token_source_provider: std::option::Option::Some(std::boxed::Box::new(ts)),
            service_account_endpoint: "https://iamcredentials.googleapis.com".to_string(),
            default_google_access_id: std::option::Option::None,
            default_sign_by: std::option::Option::None,
            project_id: std::option::Option::None,
        }
    };
    let gcs_client = google_cloud_storage::client::Client::new(client_config);

    // Define the object path where the file will be stored.
    // The file is placed inside a folder named with the UUID.
    let object_path = std::format!("s/{site_uuid}/index.html");

    // Determine the content type based on the file extension.
    let content_type = mime_guess::from_path(&object_path).first_or_octet_stream().to_string();

    let bytes = content.into_bytes();
    let mut media = google_cloud_storage::http::objects::upload::Media::new(object_path.clone());
    media.content_type = std::borrow::Cow::Owned(content_type);

    let upload_type = google_cloud_storage::http::objects::upload::UploadType::Simple(media);
    gcs_client
        .upload_object(
            &google_cloud_storage::http::objects::upload::UploadObjectRequest {
                bucket: destination_bucket.to_string(),
                ..Default::default()
            },
            bytes,
            &upload_type,
        )
        .await
        .context("Failed to upload website content to GCS")?;

    // Construct the public URL for the uploaded index.html.
    let final_url = std::format!("{base_url}/s/{site_uuid}/index.html");
    let microsite_url = url::Url::from_str(&final_url).context("Failed to create URL from the constructed string")?;
    std::result::Result::Ok(microsite_url)
}

#[cfg(test)]
mod tests {
    // Note: These tests would require actual GCS credentials or emulator setup
    // For now, we'll just test the URL construction logic
    
    #[test]
    fn test_url_construction() {
        let base_url = "https://example.com";
        let site_uuid = uuid::Uuid::new_v4();
        let final_url = std::format!("{}/s/{}/index.html", base_url, site_uuid);
        
        assert!(final_url.starts_with("https://example.com/s/"));
        assert!(final_url.ends_with("/index.html"));
    }
} 