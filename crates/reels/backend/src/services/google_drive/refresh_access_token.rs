//! Provides a function to refresh an expired Google OAuth2 access token.
//!
//! This uses the long-lived refresh token to obtain a new, short-lived access
//! token without user interaction.

/// Refreshes a Google OAuth2 access token.
///
/// # Arguments
///
/// * `refresh_token` - The refresh token to use.
/// * `http_client` - A `reqwest::Client` for making the HTTP request.
///
/// # Returns
///
/// A `Result` containing the `GoogleTokenResponse` (which will not include a new
/// refresh token) on success, or an error `String`.
pub async fn refresh_access_token(
    refresh_token: &str,
    http_client: &reqwest::Client,
) -> std::result::Result<crate::services::google_drive::models::GoogleTokenResponse, std::string::String> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| "GOOGLE_CLIENT_ID not set".to_string())?;
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .map_err(|_| "GOOGLE_CLIENT_SECRET not set".to_string())?;

    let mut params = std::collections::HashMap::new();
    params.insert("client_id", client_id.as_str());
    params.insert("client_secret", client_secret.as_str());
    params.insert("refresh_token", refresh_token);
    params.insert("grant_type", "refresh_token");

    let response = http_client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send refresh token request: {e}"))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return std::result::Result::Err(format!("Token refresh failed: {error_text}"));
    }

    response
        .json::<crate::services::google_drive::models::GoogleTokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse refresh token response: {e}"))
} 