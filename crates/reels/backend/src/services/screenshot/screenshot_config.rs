//! Configuration for screenshot services.
//!
//! This struct holds configuration data for screenshot service creation,
//! including API keys and environment detection. Uses APP_ENV to determine
//! the runtime environment (test/dev/prod) for proper service selection.
//! By reading environment variables once at startup and passing configuration
//! explicitly, we avoid environment variable race conditions in tests and follow
//! dependency injection principles for better testability and maintainability.
//!
//! Revision History:
//! - 2025-01-XX: Updated to use APP_ENV instead of STORAGE_EMULATOR_HOST for test detection
//! - 2025-01-XX: Created screenshot configuration for dependency injection refactor
//! - Prior revision history not available

/// Configuration for screenshot service creation
#[derive(Clone, Debug)]
pub struct ScreenshotConfig {
    /// Zyte API key for production screenshot service
    pub zyte_api_key: std::option::Option<std::string::String>,
    /// Whether we're running in a test environment (APP_ENV=test)
    pub is_test_environment: bool,
}

impl ScreenshotConfig {
    /// Create a new screenshot configuration
    pub fn new(zyte_api_key: std::option::Option<std::string::String>, is_test_environment: bool) -> Self {
        Self {
            zyte_api_key,
            is_test_environment,
        }
    }
    
    /// Create configuration from environment variables (called once at startup)
    pub fn from_env() -> Self {
        let zyte_api_key = std::env::var("ZYTE_API_KEY").ok();
        let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| std::string::String::from("prod"));
        let is_test_environment = app_env == "test";
        
        Self::new(zyte_api_key, is_test_environment)
    }
    
    /// Create test configuration with no API key
    pub fn for_tests() -> Self {
        Self::new(std::option::Option::None, true)
    }
    
    /// Create production configuration with provided API key
    pub fn for_production(api_key: std::string::String) -> Self {
        Self::new(std::option::Option::Some(api_key), false)
    }
}
