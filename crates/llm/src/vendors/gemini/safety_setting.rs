//! Specifies a safety setting configuration for the Gemini API request.
//!
//! Defines the category of harmful content and the threshold level to block.
//! Allows customization of content filtering behavior.
//! Uses fully qualified paths for dependencies.
//! Part of the main `Request` structure.

#[derive(Debug, serde::Serialize)]
pub struct SafetySetting {
    pub category: std::string::String,
    pub threshold: std::string::String,
}
