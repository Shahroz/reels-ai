//! Exchanges OAuth2 authorization codes for access tokens.
//!
//! Handles the token exchange step of the OAuth2 flow by sending the authorization code
//! to Google's token endpoint. Returns the access token and related token information
//! needed for subsequent API calls to fetch user information.

use oauth2::{AuthorizationCode};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use anyhow::{anyhow, Result};

/// Exchanges an authorization code for an access token.
///
/// # Arguments
///
/// * `client` - The configured OAuth2 client.
/// * `code` - The authorization code received from Google.
///
/// # Returns
///
/// A `Result` containing the token response on success.
pub async fn exchange_code(
    client: &BasicClient,
    code: AuthorizationCode,
) -> Result<oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>> {
    let start_time = std::time::Instant::now();
    let code_length = code.secret().len();
    
    log::info!(
        "Starting OAuth token exchange - Code length: {code_length}"
    );
    
    let result = client
        .exchange_code(code)
        .request_async(async_http_client)
        .await;
    
    let elapsed_ms = start_time.elapsed().as_millis();
    
    match &result {
        Ok(_) => {
            log::info!("OAuth token exchange successful - Duration: {elapsed_ms}ms");
        }
        Err(e) => {
            log::error!(
                "OAuth token exchange failed - Duration: {elapsed_ms}ms, Error details: {e:?}"
            );
            // Log additional error context if available
            match e {
                oauth2::RequestTokenError::ServerResponse(response) => {
                    log::error!("Google server error response: {response:?}");
                }
                oauth2::RequestTokenError::Request(req_err) => {
                    log::error!("OAuth request error: {req_err:?}");
                }
                oauth2::RequestTokenError::Parse(parse_err, response_body) => {
                    log::error!("OAuth response parse error: {parse_err:?}, Response body: {response_body:?}");
                }
                oauth2::RequestTokenError::Other(other) => {
                    log::error!("Other OAuth error: {other}");
                }
            }
        }
    }
    
    result.map_err(|e| anyhow!("Failed to exchange authorization code: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_code_invalid_client() {
        // Test with a minimal mock client - this will fail but we can test the error handling
        let client = BasicClient::new(
            oauth2::ClientId::new("invalid_client_id".to_string()),
            Some(oauth2::ClientSecret::new("invalid_client_secret".to_string())),
            oauth2::AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
            Some(oauth2::TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string()).unwrap()),
        );
        
        // Create a fake authorization code
        let fake_code = AuthorizationCode::new("fake_code".to_string());
        
        // This test verifies the function exists and handles errors properly
        // In a real test environment, you would mock the HTTP client
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(super::exchange_code(&client, fake_code));
        
        // Should fail with invalid credentials
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to exchange authorization code"));
    }
} 