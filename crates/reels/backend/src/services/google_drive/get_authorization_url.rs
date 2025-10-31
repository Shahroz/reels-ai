//! Provides a function to construct the Google OAuth2 authorization URL.
//!
//! This function builds the URL to which the user will be redirected to
//! grant the application permission to access their Google Drive data.

/// Constructs the Google OAuth2 authorization URL.
///
/// This URL includes the client ID, redirect URI, requested scopes, and other
/// necessary parameters for the OAuth2 "Authorization Code" flow.
///
/// # Returns
///
/// A `Result` containing the fully formed authorization URL as a `String`,
/// or an error `String` if required environment variables are missing.
pub fn get_authorization_url() -> std::result::Result<std::string::String, std::string::String> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| "GOOGLE_CLIENT_ID not set".to_string())?;
    let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
        .map_err(|_| "GOOGLE_REDIRECT_URI not set".to_string())?;

    let mut auth_url = reqwest::Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .map_err(|e| format!("Failed to parse auth URL: {e}"))?;

    auth_url.query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", "https://www.googleapis.com/auth/drive.readonly")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");

    std::result::Result::Ok(auth_url.to_string())
} 