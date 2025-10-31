//! Validates return URLs for OAuth2 redirects to prevent open redirect attacks.
//!
//! Checks that return URLs belong to allowed domains for security purposes.
//! Supports both development (localhost) and production domains for multiple deployments.
//! Returns validated URL or error for unauthorized domains.

/// Validates a return URL to ensure it belongs to an allowed domain or is a safe relative path.
///
/// # Security
///
/// - Allows relative paths (starting with `/`) - safe by design as they can't redirect externally
/// - Allows absolute URLs only from whitelisted domains
/// - Rejects absolute URLs from non-whitelisted domains
///
/// # Arguments
///
/// * `url` - The return URL to validate
///
/// # Returns
///
/// A `Result` containing the validated URL string on success, or an error message on failure.
pub fn validate_return_url(url: &str) -> Result<String, String> {
    // Allow relative paths (safe by design - can't redirect to external domains)
    if url.starts_with('/') {
        return Ok(url.to_string());
    }

    // For absolute URLs, validate the domain
    if let Ok(parsed) = url::Url::parse(url) {
        let scheme = parsed.scheme();
        let host = parsed.host_str().unwrap_or("");
        
        // Allow mobile app custom schemes
        if scheme == "realestate" {
            return Ok(url.to_string());
        }
        
        // Allow localhost for development and production domains for both deployments
        if host.starts_with("localhost") 
            || host.ends_with(".narrativ.io") 
            || host == "app.narrativ.io"
            || host == "narrativ.io"
            || host.ends_with(".bounti.ai")
            || host == "re.bounti.ai"
            || host == "bounti.ai"
            || host.ends_with(".bounti.com")
            || host == "app.bounti.com"
            || host == "bounti.com" {
            Ok(url.to_string())
        } else {
            log::warn!("Invalid return URL host rejected: {host}");
            Err(format!("Invalid return URL domain: {host}"))
        }
    } else {
        log::warn!("Invalid return URL format: {url}");
        Err("Invalid return URL format".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_return_url_relative_path() {
        let result = validate_return_url("/dashboard");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/dashboard");
    }

    #[test]
    fn test_validate_return_url_relative_path_with_query() {
        let result = validate_return_url("/real-estate/?tab=listings");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/real-estate/?tab=listings");
    }

    #[test]
    fn test_validate_return_url_relative_path_nested() {
        let result = validate_return_url("/real-estate/sign-in");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/real-estate/sign-in");
    }

    #[test]
    fn test_validate_return_url_localhost() {
        let result = validate_return_url("http://localhost:3000/path");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://localhost:3000/path");
    }

    #[test]
    fn test_validate_return_url_narrativ_domain() {
        let result = validate_return_url("https://app.narrativ.io/dashboard");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://app.narrativ.io/dashboard");
    }

    #[test]
    fn test_validate_return_url_bounti_ai_domain() {
        let result = validate_return_url("https://re.bounti.ai/real-estate");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://re.bounti.ai/real-estate");
    }

    #[test]
    fn test_validate_return_url_bounti_com_domain() {
        let result = validate_return_url("https://app.bounti.com/dashboard");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://app.bounti.com/dashboard");
    }

    #[test]
    fn test_validate_return_url_invalid_domain() {
        let result = validate_return_url("https://evil.com/steal-data");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid return URL domain"));
    }

    #[test]
    fn test_validate_return_url_invalid_format() {
        let result = validate_return_url("not-a-url");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid return URL format");
    }

    #[test]
    fn test_validate_return_url_mobile_scheme() {
        let result = validate_return_url("realestate://auth/callback");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "realestate://auth/callback");
    }
} 