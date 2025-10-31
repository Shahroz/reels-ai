//! Magic link URL construction.
//!
//! Pure function for building magic link URLs that can be unit tested
//! without requiring HTTP requests or external dependencies.

/// Builds a complete magic link URL for email.
///
/// # Arguments
///
/// * `frontend_url` - Base frontend URL (e.g., "http://localhost:5173" or "http://localhost:5173/real-estate")
/// * `token` - JWT token for magic link authentication
/// * `return_url` - Optional URL to redirect to after authentication
///
/// # Returns
///
/// Complete magic link URL string
///
/// # Examples
///
/// ```ignore
/// use backend::routes::auth::build_magic_link_url::build_magic_link_url;
///
/// // Standard deployment
/// let url = build_magic_link_url(
///     "http://localhost:5173",
///     "jwt_token_here",
///     None
/// );
/// assert_eq!(url, "http://localhost:5173/verify-login-ml?token=jwt_token_here");
///
/// // With return URL
/// let url = build_magic_link_url(
///     "http://localhost:5173",
///     "jwt_token_here",
///     Some("/dashboard")
/// );
/// assert_eq!(url, "http://localhost:5173/verify-login-ml?token=jwt_token_here&return_url=%2Fdashboard");
///
/// // Real estate deployment
/// let url = build_magic_link_url(
///     "http://localhost:5173/real-estate",
///     "jwt_token_here",
///     Some("/real-estate/listings")
/// );
/// assert_eq!(url, "http://localhost:5173/real-estate/verify-login-ml?token=jwt_token_here&return_url=%2Freal-estate%2Flistings");
/// ```
pub fn build_magic_link_url(
    frontend_url: &str,
    token: &str,
    return_url: std::option::Option<&str>,
) -> std::string::String {
    // Normalize trailing slash
    let frontend_url = frontend_url.trim_end_matches('/');
    
    // Build URL with optional return_url
    if let std::option::Option::Some(url) = return_url {
        format!(
            "{}/verify-login-ml?token={}&return_url={}",
            frontend_url,
            token,
            urlencoding::encode(url)
        )
    } else {
        format!("{}/verify-login-ml?token={}", frontend_url, token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url_without_return_url() {
        let url = build_magic_link_url(
            "http://localhost:5173",
            "test_token_123",
            std::option::Option::None,
        );
        
        assert_eq!(
            url,
            "http://localhost:5173/verify-login-ml?token=test_token_123"
        );
    }

    #[test]
    fn test_build_url_with_return_url() {
        let url = build_magic_link_url(
            "http://localhost:5173",
            "test_token_123",
            std::option::Option::Some("/dashboard"),
        );
        
        assert_eq!(
            url,
            "http://localhost:5173/verify-login-ml?token=test_token_123&return_url=%2Fdashboard"
        );
    }

    #[test]
    fn test_build_url_handles_trailing_slash() {
        let url = build_magic_link_url(
            "http://localhost:5173/",
            "test_token_123",
            std::option::Option::None,
        );
        
        assert_eq!(
            url,
            "http://localhost:5173/verify-login-ml?token=test_token_123"
        );
    }

    #[test]
    fn test_build_url_real_estate_deployment() {
        let url = build_magic_link_url(
            "http://localhost:5173/real-estate",
            "test_token_123",
            std::option::Option::Some("/real-estate/listings"),
        );
        
        assert_eq!(
            url,
            "http://localhost:5173/real-estate/verify-login-ml?token=test_token_123&return_url=%2Freal-estate%2Flistings"
        );
    }

    #[test]
    fn test_build_url_url_encodes_return_url() {
        let url = build_magic_link_url(
            "http://localhost:5173",
            "test_token_123",
            std::option::Option::Some("/path?param=value&other=123"),
        );
        
        assert!(url.contains("return_url=%2Fpath%3Fparam%3Dvalue%26other%3D123"));
    }

    #[test]
    fn test_build_url_production_domain() {
        let url = build_magic_link_url(
            "https://app.bounti.com",
            "prod_token_xyz",
            std::option::Option::Some("/"),
        );
        
        assert_eq!(
            url,
            "https://app.bounti.com/verify-login-ml?token=prod_token_xyz&return_url=%2F"
        );
    }
}

