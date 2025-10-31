//! Encodes OAuth2 state parameters for secure token transmission.
//!
//! Creates base64-encoded JSON containing CSRF token and return URL for OAuth2 flow.
//! Ensures state information is securely transmitted through the OAuth2 redirect process.
//! Used to validate and preserve user's intended destination after authentication.

use base64::Engine;
use oauth2::CsrfToken;

/// Encodes a CSRF token and return URL into a base64-encoded state parameter.
///
/// # Arguments
///
/// * `csrf_token` - The CSRF token for security validation
/// * `return_url` - The URL where the user should be redirected after OAuth2 completion
///
/// # Returns
///
/// A base64-encoded string containing the state information.
pub fn encode_oauth_state(csrf_token: &CsrfToken, return_url: &str) -> String {
    let state_data = serde_json::json!({
        "csrf": csrf_token.secret(),
        "return_url": return_url
    });
    base64::engine::general_purpose::STANDARD.encode(state_data.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_oauth_state() {
        let csrf_token = CsrfToken::new("test_csrf_token".to_string());
        let return_url = "https://app.narrativ.io/dashboard";
        
        let encoded = encode_oauth_state(&csrf_token, return_url);
        
        // Decode and verify the content
        let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(&encoded).unwrap();
        let decoded_str = String::from_utf8(decoded_bytes).unwrap();
        let decoded_json: serde_json::Value = serde_json::from_str(&decoded_str).unwrap();
        
        assert_eq!(decoded_json["csrf"], "test_csrf_token");
        assert_eq!(decoded_json["return_url"], return_url);
    }

    #[test]
    fn test_encode_oauth_state_different_urls() {
        let csrf_token = CsrfToken::new("another_token".to_string());
        let return_url = "http://localhost:3000/test";
        
        let encoded = encode_oauth_state(&csrf_token, return_url);
        
        // Verify it's properly base64 encoded
        assert!(base64::engine::general_purpose::STANDARD.decode(&encoded).is_ok());
        
        // Verify content
        let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(&encoded).unwrap();
        let decoded_str = String::from_utf8(decoded_bytes).unwrap();
        let decoded_json: serde_json::Value = serde_json::from_str(&decoded_str).unwrap();
        
        assert_eq!(decoded_json["csrf"], "another_token");
        assert_eq!(decoded_json["return_url"], return_url);
    }
} 