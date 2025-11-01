//!
//! Defines the `GeminiModel` enum and its associated methods.
//!
//! This enum represents the available Google Gemini models supported by the application.
//! It provides methods to get the model ID string, aliases, and maximum token count.
//! Adheres to the one-item-per-file guideline.
//! Extracted from `completion.rs`.

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GeminiModel {
    #[default]
    Gemini20ProExp0205,
    Gemini20Flash,
    Gemini20FlashLite,
    Gemini20FlashThinkingExp0121,
    Gemini25ProPreview0325,
    Gemini25Pro,
    Gemini25Flash,
    Gemini25FlashImage,
    Veo3,
}

impl std::fmt::Display for GeminiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}

impl GeminiModel {
    pub fn id(&self) -> &'static str {
        match self {
            GeminiModel::Gemini25ProPreview0325 => "gemini-2.5-pro-preview-03-25",
            GeminiModel::Gemini20ProExp0205 => "gemini-2.0-pro-exp-02-05",
            GeminiModel::Gemini20Flash => "gemini-2.0-flash",
            GeminiModel::Gemini20FlashLite => "gemini-2.0-flash-lite",
            GeminiModel::Gemini20FlashThinkingExp0121 => "gemini-2.0-flash-thinking-exp-01-21",
            GeminiModel::Gemini25Pro => "gemini-2.5-pro",
            GeminiModel::Gemini25Flash => "gemini-2.5-flash",
            GeminiModel::Gemini25FlashImage => "gemini-2.5-flash-image-preview",
            GeminiModel::Veo3 => "veo-3",
        }
    }

    pub fn aliases(&self) -> std::vec::Vec<&'static str> {
        match self {
            GeminiModel::Gemini25ProPreview0325 => vec!["gemini-2.5-pro", "2.5", "2.5-pro"],
            GeminiModel::Gemini20ProExp0205 => vec!["gemini-2.0-pro", "2.0-pro"],
            GeminiModel::Gemini20Flash => vec!["gemini-flash", "flash"],
            GeminiModel::Gemini20FlashLite => vec!["gemini-flash-lite", "flash-lite"],
            GeminiModel::Gemini20FlashThinkingExp0121 => vec!["gemini-flash-thinking", "flash-thinking"],
            GeminiModel::Gemini25Pro => vec![],
            GeminiModel::Gemini25Flash => vec![],
            GeminiModel::Gemini25FlashImage => vec!["nano-banana", "gemini-flash-image"],
            GeminiModel::Veo3 => vec!["veo-3", "veo3", "veo"]
        }
    }

    pub fn max_tokens(&self) -> u32 {
        match self {
            GeminiModel::Gemini20ProExp0205 => 8192,
            GeminiModel::Gemini20Flash => 8192,
            GeminiModel::Gemini20FlashLite => 8192,
            GeminiModel::Gemini20FlashThinkingExp0121 => 8192,
            GeminiModel::Gemini25ProPreview0325 => 65536,
            GeminiModel::Gemini25Pro => 65536,
            GeminiModel::Gemini25Flash => 65536,
            GeminiModel::Gemini25FlashImage => 65536,
            GeminiModel::Veo3 => 65536
        }
    }

    /// Returns true if this model is a video generation model (Veo3)
    pub fn is_video_model(&self) -> bool {
        matches!(self, GeminiModel::Veo3)
    }

    /// Returns the video generation parameters info for Veo3 models
    /// Returns None if this is not a video model
    pub fn video_params_info(&self) -> Option<Veo3VideoParams> {
        match self {
            GeminiModel::Veo3 => Some(Veo3VideoParams {
                supported_aspect_ratios: vec!["16:9", "9:16", "1:1", "4:3", "3:4"],
                max_prompt_length: 1000,
            }),
            _ => None,
        }
    }

    /// Creates a Veo3 video request from this model if it's a video model
    /// Returns an error if this is not a video model or if parameters are invalid
    pub fn create_veo3_request(
        &self,
        prompt: String,
    ) -> Result<crate::vendors::gemini::veo3_video_request::Veo3VideoRequest, String> {
        if !self.is_video_model() {
            return Err(format!("Model {:?} is not a video generation model", self));
        }

        let mut request = crate::vendors::gemini::veo3_video_request::Veo3VideoRequest::new(prompt);

        // Set default model ID for Veo3
        if matches!(self, GeminiModel::Veo3) {
            request.model = Some(self.id().to_string());
        }

        request.validate()?;
        Ok(request)
    }
}

/// Video generation parameters for Veo3 models
#[derive(Debug, Clone)]
pub struct Veo3VideoParams {
    pub supported_aspect_ratios: Vec<&'static str>,
    pub max_prompt_length: usize,
}
