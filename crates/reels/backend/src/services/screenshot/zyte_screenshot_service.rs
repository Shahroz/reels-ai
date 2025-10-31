//! Production screenshot service implementation using Zyte API.
//!
//! This service provides real screenshot functionality by integrating with
//! the Zyte API service for web page capturing. It handles API key management
//! and error transformation to maintain consistency with the screenshot service
//! interface. Used in production environments where actual screenshots are
//! required for style creation and validation workflows.
//!
//! Revision History:
//! - 2025-01-XX: Removed Debug trait to prevent API key leakage, removed from_env method
//! - Prior revision history not available

/// Production screenshot service that uses Zyte API
pub struct ZyteScreenshotService {
    api_key: std::string::String,
}

impl ZyteScreenshotService {
    /// Create a new Zyte screenshot service with the given API key
    pub fn new(api_key: std::string::String) -> Self {
        Self { api_key }
    }
    

}

#[async_trait::async_trait]
impl crate::services::screenshot::screenshot_service::ScreenshotService for ZyteScreenshotService {
    async fn screenshot_website(&self, url: &str, full_page: bool) -> std::result::Result<std::string::String, std::string::String> {
        let client = crate::zyte::zyte::ZyteClient::new(self.api_key.clone());
        client.screenshot_website(url, full_page)
            .await
            .map_err(|e| std::format!("Failed to screenshot website '{}' (full_page={}): {e}", url, full_page))
    }
}

#[cfg(test)]
mod tests {
    // Tests removed: The previous tests only verified field assignment (tautologies)
    // and provided no value. Real testing of screenshot functionality would require
    // mocking the ZyteClient, which is not implemented yet.
}