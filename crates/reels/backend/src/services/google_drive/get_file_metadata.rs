//! Provides a function to fetch metadata for a file from Google Drive.
//!
//! This function retrieves essential file details like its name, MIME type,
//! and most importantly, its version, which is used for cache invalidation.

/// Fetches metadata for a specific file from Google Drive.
///
/// This function will retry on transient failures using an exponential backoff strategy.
///
/// # Arguments
///
/// * `file_id` - The ID of the file in Google Drive.
/// * `access_token` - The user's OAuth2 access token.
/// * `http_client` - A `reqwest::Client` for making the HTTP request.
///
/// # Returns
///
/// A `Result` containing the `GoogleFileMetadata` on success, or an error `String`.
pub async fn get_file_metadata(
    file_id: &str,
    access_token: &str,
    http_client: &reqwest::Client,
) -> std::result::Result<crate::services::google_drive::models::GoogleFileMetadata, std::string::String> {
    let operation = || async {
        let url = format!("https://www.googleapis.com/drive/v3/files/{file_id}?fields=id,name,mimeType,version");

        let response = http_client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| backoff::Error::transient(format!("Request failed: {e}")))?;

        if response.status().is_client_error() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown client error".to_string());
            return std::result::Result::Err(backoff::Error::permanent(format!("Client error: {error_text}")));
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown server error".to_string());
            return std::result::Result::Err(backoff::Error::transient(format!("Server error: {error_text}")));
        }

        response
            .json::<crate::services::google_drive::models::GoogleFileMetadata>()
            .await
            .map_err(|e| backoff::Error::permanent(format!("Failed to parse metadata response: {e}")))
    };

    let backoff = backoff::ExponentialBackoff::default();
    backoff::future::retry(backoff, operation).await
} 