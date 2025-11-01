//! Defines the request structure for Google's Veo3 video generation API.
//!
//! Veo3 models require specific parameters for video generation:
//! - prompt: Text description of the video (required, max 1000 chars)
//! - Optional: durationSeconds, aspectRatio, generateAudio, enhancePrompt, negativePrompt, seed, personGeneration
//!
//! These parameters are different from text generation models.

#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Veo3VideoRequest {
    /// Text prompt describing the video to generate (required, max 1000 characters)
    pub prompt: String,
    /// Model identifier (e.g., "veo-3")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Number of videos to generate in a single request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_count: Option<u32>,
    /// Duration of the generated video in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<u32>,
    /// Aspect ratio of the video (e.g., "16:9", "9:16", "1:1")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    /// Whether to generate audio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Whether to enhance the prompt for better results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Text specifying undesired elements in the video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// Seed for deterministic output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Policy for person generation: "allow_adult" or "dont_allow"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person_generation: Option<String>,
}

impl Veo3VideoRequest {
    /// Creates a new Veo3 video generation request with required prompt
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            model: None,
            sample_count: None,
            duration_seconds: None,
            aspect_ratio: None,
            generate_audio: None,
            enhance_prompt: None,
            negative_prompt: None,
            seed: None,
            person_generation: None,
        }
    }

    /// Validates that the request has valid parameters
    pub fn validate(&self) -> Result<(), String> {
        // Validate prompt is not empty
        if self.prompt.trim().is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }

        // Validate prompt length (max 1000 characters)
        if self.prompt.len() > 1000 {
            return Err("Prompt must be 1000 characters or less".to_string());
        }

        // Validate aspect ratio if provided
        if let Some(ref aspect) = self.aspect_ratio {
            let valid_ratios = vec!["16:9", "9:16", "1:1", "4:3", "3:4"];
            if !valid_ratios.contains(&aspect.as_str()) {
                return Err(format!(
                    "Aspect ratio must be one of: {}",
                    valid_ratios.join(", ")
                ));
            }
        }

        // Validate person_generation if provided
        if let Some(ref policy) = self.person_generation {
            if !matches!(policy.as_str(), "allow_adult" | "dont_allow") {
                return Err("person_generation must be 'allow_adult' or 'dont_allow'".to_string());
            }
        }

        Ok(())
    }
}

