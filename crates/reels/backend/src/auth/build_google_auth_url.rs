//! Builds Google OAuth2 authorization URLs with custom parameters.
//!
//! Constructs the final OAuth2 URL with client ID, redirect URI, scopes, and state.
//! Handles both environment variable configuration and dynamic URL construction
//! based on request information for flexible deployment scenarios.

use actix_web::HttpRequest;

/// Builds a complete Google OAuth2 authorization URL with custom state parameter.
///
/// # Arguments
///
/// * `auth_url` - The base authorization URL from the OAuth2 client
/// * `state_encoded` - The base64-encoded state parameter containing CSRF and return URL
/// * `req` - The HTTP request for extracting connection information
///
/// # Returns
///
/// A complete OAuth2 authorization URL ready for redirection.
pub fn build_google_auth_url(
    mut auth_url: oauth2::url::Url, 
    state_encoded: &str, 
    req: &HttpRequest
) -> oauth2::url::Url {
    // Update the auth URL with our custom state
    auth_url.query_pairs_mut().clear();
    auth_url.query_pairs_mut()
        .append_pair("client_id", &std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default())
        .append_pair("redirect_uri", &std::env::var("GOOGLE_REDIRECT_URL").unwrap_or_else(|_| {
            format!("{}://{}/auth/google/callback", 
                req.connection_info().scheme(),
                req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or("localhost:8080")
            )
        }))
        .append_pair("response_type", "code")
        .append_pair("scope", "openid email profile")
        .append_pair("state", state_encoded);
    
    auth_url
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[tokio::test]
    async fn test_build_google_auth_url() {
        // Read the current environment values that the function will use
        let expected_client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
        
        // Create a test URL
        let base_url = oauth2::url::Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap();
        let state = "test_state";
        
        // Create a mock request
        let req = test::TestRequest::default()
            .append_header(("host", "localhost:8080"))
            .to_http_request();
        
        let result_url = build_google_auth_url(base_url, state, &req);
        
        let url_str = result_url.as_str();
        
        // Test that the function correctly includes the environment variable value
        if !expected_client_id.is_empty() {
            assert!(url_str.contains(&format!("client_id={}", expected_client_id)));
        }
        
        // Test that static values are correctly included
        assert!(url_str.contains("state=test_state"));
        assert!(url_str.contains("scope=openid+email+profile"));
        assert!(url_str.contains("response_type=code"));
    }

    #[tokio::test]
    async fn test_build_google_auth_url_with_state() {
        let base_url = oauth2::url::Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap();
        let state = "test_state_parameter";
        
        // Create a mock request
        let req = test::TestRequest::default()
            .append_header(("host", "localhost:8080"))
            .to_http_request();
        
        let result_url = build_google_auth_url(base_url, state, &req);
        
        let url_str = result_url.as_str();
        // Test that essential OAuth2 parameters are present
        assert!(url_str.contains("response_type=code"));
        assert!(url_str.contains("scope=openid+email+profile"));
        assert!(url_str.contains("state=test_state_parameter"));
        assert!(url_str.contains("redirect_uri="));
        // Test that client_id parameter is present (regardless of value)
        assert!(url_str.contains("client_id="));
    }
} 