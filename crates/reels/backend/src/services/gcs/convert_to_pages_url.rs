//! Converts a GCS storage URL to a pages.bounti.ai URL for the bounti_prod_narrativ_public bucket.
//!
//! This utility function helps convert GCS URLs to the pages.bounti.ai domain for public access.
//! Only URLs from the bounti_prod_narrativ_public bucket are converted.

/// Converts a GCS storage URL to a pages.bounti.ai URL for the bounti_prod_narrativ_public bucket.
/// 
/// # Arguments
/// * `gcs_url` - The GCS URL to convert
/// 
/// # Returns
/// The converted pages.bounti.ai URL, or the original URL if it's not a bounti_prod_narrativ_public GCS URL
/// 
/// # Examples
/// ```ignore
/// use crate::services::gcs::convert_to_pages_url::convert_to_pages_url;
/// 
/// let gcs_url = "https://storage.googleapis.com/bounti_prod_narrativ_public/creatives/123/creative.html";
/// let pages_url = convert_to_pages_url(gcs_url);
/// assert_eq!(pages_url, "https://pages.bounti.ai/creatives/123/creative.html");
/// 
/// // Non-bounti_prod_narrativ_public URLs are returned unchanged
/// let other_url = "https://storage.googleapis.com/other_bucket/file.html";
/// let unchanged = convert_to_pages_url(other_url);
/// assert_eq!(unchanged, other_url);
/// ```
pub fn convert_to_pages_url(gcs_url: &str) -> std::string::String {
    if let Some(path) = gcs_url.strip_prefix("https://storage.googleapis.com/bounti_prod_narrativ_public/") {
        std::format!("https://pages.bounti.ai/{path}")
    } else {
        // Return original URL if it's not a bounti_prod_narrativ_public GCS URL
        gcs_url.to_string()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_convert_bounti_prod_narrativ_public_url() {
        let gcs_url = "https://storage.googleapis.com/bounti_prod_narrativ_public/creatives/123/creative.html";
        let pages_url = super::convert_to_pages_url(gcs_url);
        assert_eq!(pages_url, "https://pages.bounti.ai/creatives/123/creative.html");
    }

    #[test]
    fn test_convert_other_bucket_url_unchanged() {
        let other_url = "https://storage.googleapis.com/other_bucket/file.html";
        let unchanged = super::convert_to_pages_url(other_url);
        assert_eq!(unchanged, other_url);
    }

    #[test]
    fn test_convert_non_gcs_url_unchanged() {
        let other_url = "https://example.com/file.html";
        let unchanged = super::convert_to_pages_url(other_url);
        assert_eq!(unchanged, other_url);
    }
} 