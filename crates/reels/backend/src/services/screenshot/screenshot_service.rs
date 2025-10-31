//! Trait for taking screenshots of web pages.
//!
//! This trait provides a common interface for screenshot functionality,
//! enabling dependency injection and testability in the application.
//! Implementations can vary from production services using external APIs
//! to mock services returning dummy data for testing scenarios.
//! The trait is async and thread-safe for use in concurrent environments.

/// Trait for taking screenshots of web pages
#[async_trait::async_trait]
pub trait ScreenshotService: Send + Sync {
    /// Take a screenshot of the given URL
    /// 
    /// # Arguments
    /// * `url` - The URL to screenshot
    /// * `full_page` - Whether to capture the full page or just viewport
    /// 
    /// # Returns
    /// Base64-encoded screenshot data on success, error message on failure
    async fn screenshot_website(&self, url: &str, full_page: bool) -> std::result::Result<std::string::String, std::string::String>;
}