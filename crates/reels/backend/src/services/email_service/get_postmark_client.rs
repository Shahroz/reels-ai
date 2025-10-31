//! Postmark client initialization.
//!
//! Creates a Postmark email client using the POSTMARK_SERVER_TOKEN environment variable.
//! Returns an error if the token is not configured.

/// Creates a Postmark client for sending emails.
///
/// # Returns
///
/// A `Result` containing the `PostmarkClient` on success, or an error if
/// the POSTMARK_SERVER_TOKEN environment variable is not set.
pub fn get_postmark_client() -> anyhow::Result<postmark::reqwest::PostmarkClient> {
    let api_key = std::env::var("POSTMARK_SERVER_TOKEN")
        .map_err(|_| anyhow::anyhow!("POSTMARK_SERVER_TOKEN environment variable not set"))?;
    
    std::result::Result::Ok(postmark::reqwest::PostmarkClient::builder()
        .server_token(api_key)
        .build())
}

// Note: This function should be called once at startup and cached in app state.
// Testing will be done via integration tests with POSTMARK_SERVER_TOKEN configured.
// See: TODO - Move to singleton pattern for performance

