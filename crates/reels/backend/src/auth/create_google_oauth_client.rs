//! Creates a configured Google OAuth2 client for authentication.
//!
//! Initializes the OAuth2 client with credentials from environment variables.
//! Handles client ID, secret, and redirect URL configuration for Google OAuth2 flow.
//! Returns a configured BasicClient ready for authorization URL generation and token exchange.

use oauth2::{ClientId, ClientSecret, RedirectUrl};
use oauth2::basic::BasicClient;
use anyhow::{anyhow, Result};
use crate::auth::constants::{GOOGLE_AUTH_URL, GOOGLE_TOKEN_URL, DEFAULT_GOOGLE_REDIRECT_URL};

/// Creates a configured Google OAuth2 client.
///
/// # Returns
///
/// A `Result` containing a configured `BasicClient` on success, or an error if
/// the required environment variables are not set.
pub fn create_google_oauth_client() -> Result<BasicClient> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| anyhow!("GOOGLE_CLIENT_ID environment variable not set"))?;
    
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .map_err(|_| anyhow!("GOOGLE_CLIENT_SECRET environment variable not set"))?;
    
    let redirect_url = std::env::var("GOOGLE_REDIRECT_URL")
        .unwrap_or_else(|_| DEFAULT_GOOGLE_REDIRECT_URL.to_string());

    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        oauth2::AuthUrl::new(GOOGLE_AUTH_URL.to_string())
            .map_err(|e| anyhow!("Invalid auth URL: {}", e))?,
        Some(
            oauth2::TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
                .map_err(|e| anyhow!("Invalid token URL: {}", e))?,
        ),
    )
    .set_redirect_uri(
        RedirectUrl::new(redirect_url)
            .map_err(|e| anyhow!("Invalid redirect URL: {}", e))?,
    );

    Ok(client)
}

 