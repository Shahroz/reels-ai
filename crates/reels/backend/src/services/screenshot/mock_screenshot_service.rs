//! Mock screenshot service implementation for testing.
//!
//! This service provides a fake screenshot implementation that returns
//! a valid but minimal PNG image encoded as base64. Used exclusively in
//! test environments to avoid external API dependencies and network calls.
//! The returned data is a 1x1 transparent PNG that satisfies image validation
//! requirements while maintaining fast and reliable test execution.

/// Mock screenshot service that returns a dummy screenshot for testing
pub struct MockScreenshotService;

impl MockScreenshotService {
    /// Create a new mock screenshot service
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockScreenshotService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl crate::services::screenshot::screenshot_service::ScreenshotService for MockScreenshotService {
    async fn screenshot_website(&self, _url: &str, _full_page: bool) -> std::result::Result<std::string::String, std::string::String> {
        // Return a tiny 1x1 transparent PNG encoded as base64
        // This is valid PNG data that can be decoded and used in tests
        std::result::Result::Ok("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==".to_string())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_screenshot_website_returns_valid_base64_png() {
        let service = super::MockScreenshotService::new();
        let result = <super::MockScreenshotService as crate::services::screenshot::screenshot_service::ScreenshotService>::screenshot_website(&service, "http://example.com", true).await;
        std::assert!(result.is_ok());
        let base64_data = result.unwrap();
        // Verify it's valid base64 by attempting to decode  
        let decoded = <base64::engine::GeneralPurpose as base64::Engine>::decode(&base64::engine::general_purpose::STANDARD, base64_data);
        std::assert!(decoded.is_ok());
    }

    #[tokio::test]
    async fn test_screenshot_website_returns_consistent_results() {
        let service = super::MockScreenshotService::new();
        let result1 = <super::MockScreenshotService as crate::services::screenshot::screenshot_service::ScreenshotService>::screenshot_website(&service, "http://example.com", true).await;
        let result2 = <super::MockScreenshotService as crate::services::screenshot::screenshot_service::ScreenshotService>::screenshot_website(&service, "http://different.com", false).await;
        // Mock service should return consistent results regardless of parameters
        std::assert!(result1.is_ok());
        std::assert!(result2.is_ok());
        std::assert_eq!(result1.unwrap(), result2.unwrap());
    }
}