//! Factory for creating screenshot services with dependency injection.
//!
//! This factory function creates the appropriate screenshot service implementation
//! based on explicit configuration rather than environment detection. This approach
//! eliminates environment variable race conditions and follows dependency injection
//! principles for better testability and maintainability. Configuration is read
//! once at startup and passed explicitly to avoid global state dependencies.
//!
//! Revision History:
//! - 2025-01-XX: Refactored to use dependency injection instead of environment detection
//! - Prior revision history not available

/// Creates the appropriate screenshot service based on configuration
pub fn create_screenshot_service(
    config: &crate::services::screenshot::screenshot_config::ScreenshotConfig,
) -> std::result::Result<std::sync::Arc<dyn crate::services::screenshot::screenshot_service::ScreenshotService>, std::string::String> {
    if config.is_test_environment {
        // Test environment - use mock service
        std::result::Result::Ok(std::sync::Arc::new(crate::services::screenshot::mock_screenshot_service::MockScreenshotService::new()))
    } else {
        // Production environment - use real Zyte service
        match &config.zyte_api_key {
            std::option::Option::Some(api_key) => {
                let service = crate::services::screenshot::zyte_screenshot_service::ZyteScreenshotService::new(api_key.clone());
                std::result::Result::Ok(std::sync::Arc::new(service) as std::sync::Arc<dyn crate::services::screenshot::screenshot_service::ScreenshotService>)
            }
            std::option::Option::None => {
                std::result::Result::Err(std::string::String::from("ZYTE_API_KEY not provided in configuration"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_creates_mock_service_in_test_environment() {
        let config = crate::services::screenshot::screenshot_config::ScreenshotConfig::for_tests();
        let result = super::create_screenshot_service(&config);
        std::assert!(result.is_ok());
    }

    #[test]
    fn test_creates_zyte_service_with_api_key() {
        let config = crate::services::screenshot::screenshot_config::ScreenshotConfig::for_production(std::string::String::from("test_api_key"));
        let result = super::create_screenshot_service(&config);
        std::assert!(result.is_ok());
    }

    #[test]
    fn test_fails_without_api_key_in_production() {
        let config = crate::services::screenshot::screenshot_config::ScreenshotConfig::new(std::option::Option::None, false);
        let result = super::create_screenshot_service(&config);
        std::assert!(result.is_err());
        let error_msg = result.err().unwrap();
        std::assert_eq!(error_msg, std::string::String::from("ZYTE_API_KEY not provided in configuration"));
    }


}