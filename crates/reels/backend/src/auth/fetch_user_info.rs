//! Fetches user information from Google OAuth2 API.
//!
//! Uses the access token obtained from OAuth2 flow to retrieve user profile information
//! from Google's userinfo endpoint. Returns user data including email, name, and profile
//! information as JSON for account creation or login.

use anyhow::{anyhow, Result};

/// Fetches user information from Google using the access token.
///
/// # Arguments
///
/// * `access_token` - The access token obtained from the OAuth2 flow.
///
/// # Returns
///
/// A `Result` containing the user information as a JSON value.
pub async fn fetch_user_info(access_token: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to fetch user info: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch user info: HTTP {}", response.status()));
    }

    let user_info = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse user info: {}", e))?;

    Ok(user_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_user_info_invalid_token() {
        // Test with an invalid token - this will fail but we can test the error handling
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(super::fetch_user_info("invalid_token"));
        
        // Should fail with invalid token
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to fetch user info"));
    }

    #[test]
    fn test_fetch_user_info_empty_token() {
        // Test with empty token
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(super::fetch_user_info(""));
        
        // Should fail with empty token
        assert!(result.is_err());
    }
} 