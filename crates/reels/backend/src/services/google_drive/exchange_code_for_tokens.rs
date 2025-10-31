//! Provides a function to exchange a Google OAuth2 authorization code for tokens.
//!
//! This function is called from the OAuth2 callback handler. It sends the
//! authorization code to Google's token endpoint to receive access and refresh tokens.

/// Exchanges an authorization code for Google OAuth2 tokens.
///
/// # Arguments
///
/// * `code` - The authorization code received from Google's callback.
/// * `http_client` - A `reqwest::Client` for making the HTTP request.
///
/// # Returns
///
/// A `Result` containing the `GoogleTokenResponse` on success, or an error `String`.
pub async fn exchange_code_for_tokens(
    code: &str,
    http_client: &reqwest::Client,
) -> std::result::Result<crate::services::google_drive::models::GoogleTokenResponse, std::string::String> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| "GOOGLE_CLIENT_ID not set".to_string())?;
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .map_err(|_| "GOOGLE_CLIENT_SECRET not set".to_string())?;
    let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
        .map_err(|_| "GOOGLE_REDIRECT_URI not set".to_string())?;

    let mut params = std::collections::HashMap::new();
    params.insert("client_id", client_id.as_str());
    params.insert("client_secret", client_secret.as_str());
    params.insert("code", code);
    params.insert("grant_type", "authorization_code");
    params.insert("redirect_uri", redirect_uri.as_str());

    let response = http_client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send token request: {e}"))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return std::result::Result::Err(format!("Token exchange failed: {error_text}"));
    }

    response
        .json::<crate::services::google_drive::models::GoogleTokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse token response: {e}"))
} 