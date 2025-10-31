//! Provides a function to revoke a Google OAuth2 token.
//!
//! This function is used when a user wants to disconnect their Google account
//! from the application, ensuring that the application can no longer access
//! their data.

/// Revokes a Google OAuth2 token.
///
/// This is typically used with a refresh token to invalidate it permanently.
///
/// # Arguments
///
/// * `token` - The token to revoke (usually the refresh token).
/// * `http_client` - A `reqwest::Client` for making the HTTP request.
///
/// # Returns
///
/// An empty `Result` on success, or an error `String` on failure.
pub async fn revoke_token(
    token: &str,
    http_client: &reqwest::Client,
) -> std::result::Result<(), std::string::String> {
    let mut params = std::collections::HashMap::new();
    params.insert("token", token);

    let response = http_client
        .post("https://oauth2.googleapis.com/revoke")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send revoke request: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        // We don't return an error for a 400 status, as it often means the token
        // was already invalid, which is an acceptable state for revocation.
        if status.as_u16() != 400 {
            return std::result::Result::Err(format!("Token revocation failed: {error_text}"));
        }
    }

    std::result::Result::Ok(())
} 