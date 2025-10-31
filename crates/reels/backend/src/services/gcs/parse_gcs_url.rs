//! Parses GCS URLs in multiple formats and returns bucket and object names.
//!
//! Supports both gs:// and https://storage.googleapis.com/ URL formats, plus pages.bounti.ai URLs.
//! This utility function helps standardize GCS URL parsing across the application.

/// Parses GCS URLs in multiple formats and returns bucket and object names.
/// 
/// Supports both gs:// and https://storage.googleapis.com/ URL formats, plus pages.bounti.ai URLs.
/// 
/// # Arguments
/// * `gcs_url` - The GCS URL to parse
/// 
/// # Returns
/// A Result containing a tuple of (bucket_name, object_name) on success,
/// or an error message on failure.
/// 
/// # Examples
/// ```ignore
/// use crate::services::gcs::parse_gcs_url::parse_gcs_url;
/// 
/// let (bucket, object) = parse_gcs_url("gs://my-bucket/path/to/file.txt").unwrap();
/// assert_eq!(bucket, "my-bucket");
/// assert_eq!(object, "path/to/file.txt");
/// 
/// let (bucket, object) = parse_gcs_url("https://storage.googleapis.com/my-bucket/path/to/file.txt").unwrap();
/// assert_eq!(bucket, "my-bucket");
/// assert_eq!(object, "path/to/file.txt");
/// 
/// let (bucket, object) = parse_gcs_url("https://pages.bounti.ai/creatives/123/creative.html").unwrap();
/// assert_eq!(bucket, "bounti_prod_narrativ_public");
/// assert_eq!(object, "creatives/123/creative.html");
/// ```
pub fn parse_gcs_url(gcs_url: &str) -> std::result::Result<(std::string::String, std::string::String), std::string::String> {
    if let Some(path) = gcs_url.strip_prefix("gs://") {
        let parts: std::vec::Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            std::result::Result::Ok((parts[0].to_string(), parts[1].to_string()))
        } else {
            std::result::Result::Err("Invalid gs:// URL format: expected gs://bucket/object".to_string())
        }
    } else if let Some(path) = gcs_url.strip_prefix("https://storage.googleapis.com/") {
        let parts: std::vec::Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            std::result::Result::Ok((parts[0].to_string(), parts[1].to_string()))
        } else {
            std::result::Result::Err("Invalid https://storage.googleapis.com/ URL format: expected https://storage.googleapis.com/bucket/object".to_string())
        }
    } else if let Some(path) = gcs_url.strip_prefix("https://pages.bounti.ai/") {
        // pages.bounti.ai URLs map to the bounti_prod_narrativ_public bucket
        // Example: https://pages.bounti.ai/creatives/123/creative.html -> bucket: bounti_prod_narrativ_public, object: creatives/123/creative.html
        if !path.is_empty() {
            std::result::Result::Ok(("bounti_prod_narrativ_public".to_string(), path.to_string()))
        } else {
            std::result::Result::Err("Invalid https://pages.bounti.ai/ URL format: expected https://pages.bounti.ai/object".to_string())
        }
    } else {
        std::result::Result::Err("Unsupported URL scheme. Must be gs://, https://storage.googleapis.com/, or https://pages.bounti.ai/".to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_gs_url() {
        let (bucket, object) = super::parse_gcs_url("gs://my-bucket/path/to/file.txt").unwrap();
        assert_eq!(bucket, "my-bucket");
        assert_eq!(object, "path/to/file.txt");
    }

    #[test]
    fn test_parse_storage_googleapis_url() {
        let (bucket, object) = super::parse_gcs_url("https://storage.googleapis.com/my-bucket/path/to/file.txt").unwrap();
        assert_eq!(bucket, "my-bucket");
        assert_eq!(object, "path/to/file.txt");
    }

    #[test]
    fn test_parse_pages_bounti_url() {
        let (bucket, object) = super::parse_gcs_url("https://pages.bounti.ai/creatives/123/creative.html").unwrap();
        assert_eq!(bucket, "bounti_prod_narrativ_public");
        assert_eq!(object, "creatives/123/creative.html");
    }

    #[test]
    fn test_parse_invalid_url() {
        let result = super::parse_gcs_url("https://example.com/file.txt");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unsupported URL scheme. Must be gs://, https://storage.googleapis.com/, or https://pages.bounti.ai/");
    }

    #[test]
    fn test_parse_invalid_gs_url() {
        let result = super::parse_gcs_url("gs://bucket-without-object");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid gs:// URL format: expected gs://bucket/object");
    }
} 