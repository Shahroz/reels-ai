//! Defines the request structure for OpenAI's Sora video generation API.
//!
//! Sora models require specific parameters for video generation:
//! - prompt: Text description of the video
//! - size: Video resolution (e.g., "1280x720", "720x1280")
//! - seconds: Video duration in seconds ("4", "8", or "12")
//!
//! These parameters are different from chat completion models.

#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SoraVideoRequest {
    /// Text prompt describing the video to generate (required)
    pub prompt: String,
    /// Model identifier (e.g., "sora-1.0")
    pub model: String,
    /// Video resolution in format "WIDTHxHEIGHT" (required)
    /// Supported sizes: "1280x720", "720x1280" for sora-1.0
    pub size: String,
    /// Video duration in seconds (required)
    /// Supported values: "4", "8", "12"
    /// Default: "4"
    #[serde(default = "default_seconds")]
    pub seconds: String,
}

fn default_seconds() -> String {
    "4".to_string()
}

impl SoraVideoRequest {
    /// Creates a new Sora video generation request with required parameters
    pub fn new(prompt: String, model: String, size: String, seconds: Option<String>) -> Self {
        Self {
            prompt,
            model,
            size,
            seconds: seconds.unwrap_or_else(|| "4".to_string()),
        }
    }

    /// Validates that the request has valid parameters
    pub fn validate(&self) -> Result<(), String> {
        // Validate size format
        if !self.size.contains('x') {
            return Err("Size must be in format 'WIDTHxHEIGHT' (e.g., '1280x720')".to_string());
        }

        // Validate seconds
        if !matches!(self.seconds.as_str(), "4" | "8" | "12") {
            return Err("Seconds must be '4', '8', or '12'".to_string());
        }

        // Validate prompt is not empty
        if self.prompt.trim().is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }

        Ok(())
    }
}

