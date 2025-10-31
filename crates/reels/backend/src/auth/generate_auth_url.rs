//! Generates authorization URLs for Google OAuth2 login.
//!
//! Creates the initial authorization URL that redirects users to Google's OAuth2 server.
//! Configures required scopes (openid, email, profile) and CSRF protection tokens.
//! Returns both the authorization URL and the CSRF token for validation.

use oauth2::{CsrfToken, Scope};
use oauth2::basic::BasicClient;

/// Generates an authorization URL for Google OAuth2 login.
///
/// # Arguments
///
/// * `client` - The configured OAuth2 client.
///
/// # Returns
///
/// A tuple containing the authorization URL and CSRF token.
pub fn generate_auth_url(client: &BasicClient) -> (oauth2::url::Url, CsrfToken) {
    client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::create_google_oauth_client::create_google_oauth_client;

    #[test]
    fn test_generate_auth_url() {
        // Only run this test if the required environment variables are set
        let expected_client_id = match std::env::var("GOOGLE_CLIENT_ID") {
            Ok(id) => id,
            Err(_) => {
                // Skip test if environment variables aren't set
                println!("Skipping test_generate_auth_url: GOOGLE_CLIENT_ID not set");
                return;
            }
        };
        
        if std::env::var("GOOGLE_CLIENT_SECRET").is_err() {
            println!("Skipping test_generate_auth_url: GOOGLE_CLIENT_SECRET not set");
            return;
        }
        
        // Test the function with the actual environment values
        let client = create_google_oauth_client().expect("Client creation should succeed");
        let (auth_url, _csrf_token) = super::generate_auth_url(&client);
        
        let url_str = auth_url.as_str();
        assert!(url_str.contains("https://accounts.google.com/o/oauth2/v2/auth"));
        assert!(url_str.contains(&format!("client_id={}", expected_client_id)));
        assert!(url_str.contains("scope=openid+email+profile"));
    }
} 