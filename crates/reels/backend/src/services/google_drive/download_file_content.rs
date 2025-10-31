//! Provides a function to download the content of a file from Google Drive.
//!
//! This function handles two cases: standard files are downloaded directly, while
//! Google Workspace documents (Docs, Sheets, Slides) are exported as PDFs.

/// Downloads the content of a file from Google Drive.
///
/// This function will retry on transient failures using an exponential backoff strategy.
///
/// # Arguments
///
/// * `file_id` - The ID of the file to download.
/// * `mime_type` - The MIME type of the file, used to determine if export is needed.
/// * `access_token` - The user's OAuth2 access token.
/// * `http_client` - A `reqwest::Client` for making the HTTP request.
///
/// # Returns
///
/// A `Result` containing the file content as `Vec<u8>` on success, or an error `String`.
pub async fn download_file_content(
    file_id: &str,
    mime_type: &str,
    access_token: &str,
    http_client: &reqwest::Client,
) -> std::result::Result<std::vec::Vec<u8>, std::string::String> {
    let operation = || async {
        let url = if mime_type.starts_with("application/vnd.google-apps") {
            // It's a Google Workspace doc, so we need to export it.
            // We'll export as PDF, which is a good format for text extraction.
            let export_mime_type = "application/pdf";
            format!("https://www.googleapis.com/drive/v3/files/{file_id}/export?mimeType={export_mime_type}")
        } else {
            // It's a standard file, download it directly.
            format!("https://www.googleapis.com/drive/v3/files/{file_id}?alt=media")
        };

        let response = http_client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| backoff::Error::transient(format!("Request failed: {e}")))?;

        if response.status().is_client_error() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown client error".to_string());
            return std::result::Result::Err(backoff::Error::permanent(format!("Client error on download: {error_text}")));
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown server error".to_string());
            return std::result::Result::Err(backoff::Error::transient(format!("Server error on download: {error_text}")));
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| backoff::Error::permanent(format!("Failed to read file content bytes: {e}")))
    };

    let backoff = backoff::ExponentialBackoff::default();
    backoff::future::retry(backoff, operation).await
} 