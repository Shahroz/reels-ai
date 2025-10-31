//! Result type for screenshot operations.
//!
//! This type alias provides a standardized return type for all screenshot
//! service implementations, ensuring consistency across the codebase.
//! Returns either base64-encoded screenshot data on success or an error
//! message string on failure. The design prioritizes simplicity and
//! clarity over complex error types for this specific use case.

/// Result type for screenshot operations
pub type ScreenshotResult = std::result::Result<std::string::String, std::string::String>;