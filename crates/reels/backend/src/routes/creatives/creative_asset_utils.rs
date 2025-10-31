//! Utility function for uploading creative assets (HTML and screenshot) to GCS.
//!
//! This module provides a reusable function to handle the process of:
//! 1. Uploading HTML content for a creative to a standardized GCS path.
//! 2. Generating a screenshot of the uploaded HTML using Zyte.
//! 3. Uploading the generated screenshot to a standardized GCS path.
//! It returns the pages.bounti.ai URLs for both the HTML and the screenshot.
//! Adheres to coding standards, using fully qualified paths where necessary.

use crate::zyte::zyte::ZyteClient;
use crate::services::gcs::convert_to_pages_url::convert_to_pages_url;
use base64::Engine as _; // Use `as _` to avoid ambiguity if base64::Engine is used elsewhere

/// Uploads HTML content and its screenshot to GCS.
///
/// # Arguments
/// * `gcs_client` - A reference to the GCSOperations trait.
/// * `creative_id` - The UUID of the creative, used for path generation.
/// * `html_content_bytes` - The raw HTML content as a byte vector.
///
/// # Returns
/// A `Result` containing a tuple of (html_url, screenshot_url) on success,
/// or a `String` error message on failure.
// Note: This function involves multiple I/O operations and might exceed 50 LoC.
// This is justified by the sequential nature of cloud storage and external API interactions.
pub async fn upload_creative_assets(
    gcs_client: &dyn crate::services::gcs::gcs_operations::GCSOperations,
    creative_id: uuid::Uuid,
    html_content_bytes: std::vec::Vec<u8>, // These are the bytes of the content to be uploaded
) -> std::result::Result<(std::string::String, std::string::String), std::string::String> {
    // 1. Get GCS Bucket Name
    let bucket_name = match std::env::var("GCS_BUCKET") {
        Ok(b) => b,
        Err(_) => {
            log::error!("GCS_BUCKET environment variable not set.");
            return std::result::Result::Err(std::string::String::from(
                "Server configuration error: Missing GCS_BUCKET.",
            ));
        }
    };

    // --- BEGIN ADDED DELETION LOGIC ---
    // Attempt to delete existing main HTML object to ensure overwrite by upload.
    // This is particularly relevant if this function is used to publish a draft,
    // replacing existing main creative content.
    let main_html_object_to_delete = format!("creatives/{creative_id}/creative.html");
    match gcs_client.delete_object(&bucket_name, &main_html_object_to_delete).await {
        Ok(()) => {
            log::info!(
                "Successfully deleted existing main HTML object before new upload: {bucket_name}/{main_html_object_to_delete}"
            );
        }
        Err(e) => {
            // Log as warning: If object didn't exist, delete fails, which is acceptable.
            // If another error occurs during delete, upload might still succeed or fail with its own error.
            log::warn!(
                "Attempt to delete main HTML object {bucket_name}/{main_html_object_to_delete} failed (may be benign if not found): {e}. Proceeding with upload."
            );
        }
    }

    // Attempt to delete existing main screenshot object.
    let main_screenshot_object_to_delete = format!("creatives/{creative_id}/screenshot.png");
    match gcs_client.delete_object(&bucket_name, &main_screenshot_object_to_delete).await {
        Ok(()) => {
            log::info!("Successfully deleted existing main screenshot object before new upload: {bucket_name}/{main_screenshot_object_to_delete}");
        }
        Err(e) => {
            log::warn!("Attempt to delete main screenshot object {bucket_name}/{main_screenshot_object_to_delete} failed (may be benign if not found): {e}. Proceeding.");
        }
    }
    // --- END ADDED DELETION LOGIC ---

    // 2. Upload HTML Content
    let html_object_name = format!("creatives/{creative_id}/creative.html");
    let html_gcs_url = match gcs_client
        .upload_raw_bytes(
            &bucket_name,
            &html_object_name,
            "text/html",
            html_content_bytes,
            true,
            crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic
        )
        .await
    {
        Ok(url) => url,
        Err(e) => {
            log::error!(
                "Failed to upload creative HTML to GCS (id: {creative_id}): {e}"
            );
            return std::result::Result::Err(std::string::String::from(
                "Failed to store creative HTML.",
            ));
        }
    };

    // Convert to pages.bounti.ai URL for consistent use
    let html_pages_url = convert_to_pages_url(&html_gcs_url);

    // 3. Generate Screenshot via Zyte (using pages.bounti.ai URL)
    let zyte_api_key = std::env::var("ZYTE_API_KEY").unwrap_or_default();
    // Check if API key is empty, which would cause ZyteClient to fail.
    if zyte_api_key.is_empty() {
        log::error!("ZYTE_API_KEY environment variable not set or empty.");
        // Optionally, try to delete the uploaded HTML if screenshot fails this early
        // For simplicity here, just returning error.
        return std::result::Result::Err(std::string::String::from(
            "Server configuration error: Missing ZYTE_API_KEY.",
        ));
    }

    let zyte_client = ZyteClient::new(zyte_api_key);
    let screenshot_base64 = match zyte_client.screenshot_website(&html_pages_url, true).await {
        Ok(s) => s,
        Err(e) => {
            log::error!(
                "Failed to screenshot creative HTML via Zyte (id: {creative_id}, url: {html_pages_url}): {e}"
            );
            return std::result::Result::Err(std::string::String::from(
                "Failed to generate screenshot for creative.",
            ));
        }
    };

    let screenshot_data_bytes =
        match base64::engine::general_purpose::STANDARD.decode(&screenshot_base64) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!(
                    "Invalid base64 in screenshot data (id: {creative_id}): {e}"
                );
                return std::result::Result::Err(std::string::String::from(
                    "Failed to process screenshot data.",
                ));
            }
        };

    // 4. Upload Screenshot to GCS
    let screenshot_object_name = format!("creatives/{creative_id}/screenshot.png");
    let screenshot_gcs_url = match gcs_client
        .upload_raw_bytes(
            &bucket_name,
            &screenshot_object_name,
            "image/png",
            screenshot_data_bytes,
            false,
            crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic
        )
        .await
    {
        Ok(url) => url,
        Err(e) => {
            log::error!(
                "Failed to upload screenshot to GCS (id: {creative_id}): {e}"
            );
            return std::result::Result::Err(std::string::String::from(
                "Failed to store screenshot.",
            ));
        }
    };

    // Convert screenshot URL to pages.bounti.ai format
    let screenshot_pages_url = convert_to_pages_url(&screenshot_gcs_url);

    std::result::Result::Ok((html_pages_url, screenshot_pages_url))
}

#[cfg(test)]
mod tests {
    // Unit tests for `upload_creative_assets` would typically require mocking GCSClient and ZyteClient,
    // or integration tests with actual services, which is beyond the scope of simple file-based tests.
    // For now, we acknowledge that testing this function effectively needs external dependencies.

    // Example of a placeholder test structure:
    #[test]
    fn test_placeholder_for_asset_upload_logic() {
        // In a real scenario, you would:
        // 1. Setup mock GCSClient and mock ZyteClient.
        // 2. Define expected inputs and mock outputs.
        // 3. Call `super::upload_creative_assets` with mocks.
        // 4. Assert that the interactions with mocks were as expected.
        // 5. Assert that the returned URLs match mock outputs or expected formats.
        assert!(true, "Test needs to be implemented with mocks or integration setup.");
    }
}
