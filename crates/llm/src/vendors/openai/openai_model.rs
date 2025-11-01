//! Defines the available OpenAI model identifiers.
//! 
//! Includes various GPT models like GPT-4o, GPT-4.1, etc.
//! Provides serialization names matching the API and a default model.
//! Also includes aliases for user convenience.
//! Used to specify the model in OpenAIChatCompletionRequest.

use crate::vendors::openai::reasoning_effort::ReasoningEffort;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Default)]
pub enum OpenAIModel {
    #[serde(rename = "gpt-4o-2024-11-20")]
    Gpt4o20241120,
    #[serde(rename = "gpt-4o-mini-2024-07-18")]
    #[default]
    Gpt4oMini20240718,
    #[serde(rename = "o3-mini")]
    Gpto3Mini,
    #[serde(rename = "gpt-4.1-2025-04-14")]
    Gpt4120250414,
    #[serde(rename = "gpt-4.1-mini-2025-04-14")]
    Gpt41Mini20250414,
    #[serde(rename = "gpt-4.1-nano-2025-04-14")]
    Gpt41Nano20250414,
    #[serde(rename = "o4-mini-2025-04-16")]
    GptO4Mini20250416,
    #[serde(rename = "o3-2025-04-16")]
    GptO320250416,
    #[serde(rename = "sora-1.0")]
    Sora10,
}

impl OpenAIModel {
    //! Provides aliases for OpenAI model identifiers.
    //! 
    //! Returns a vector of string slices representing common aliases
    //! for each model variant, useful for parsing user input.
    //! Helps map friendly names to the canonical model enum.
    //! Example: "4o" maps to Gpt4o20241120.
    pub fn aliases(&self) -> Vec<&'static str> {
        match self {
            OpenAIModel::Gpt4o20241120 => vec!["gpt-4o-2024-11-20", "gpt-4o", "4o-1120"],
            OpenAIModel::Gpt4oMini20240718 => vec!["gpt-4o-mini-2024-07-18", "gpt-4o-mini", "gpt4omini", "4omini"],
            OpenAIModel::Gpto3Mini => vec!["o3-mini", "gpt-o3-mini", "o3m"],
            OpenAIModel::Gpt4120250414 => vec!["gpt-4.1-2025-04-14", "gpt-4.1", "4.1"],
            OpenAIModel::Gpt41Mini20250414 => vec!["gpt-4.1-mini-2025-04-14", "gpt-4.1-mini", "4.1m"],
            OpenAIModel::Gpt41Nano20250414 => vec!["gpt-4.1-nano-2025-04-14", "gpt-4.1-nano", "4.1n"],
            OpenAIModel::GptO4Mini20250416 => vec!["o4"],
            OpenAIModel::GptO320250416 => vec!["o3"],
            OpenAIModel::Sora10 => vec!["sora-1.0", "sora", "sora1.0"]
        }
    }

    pub fn reasoning_effort(&self) -> Option<ReasoningEffort> {
        match self {
            OpenAIModel::Gpt4o20241120 => None,
            OpenAIModel::Gpt4oMini20240718 => None,
            OpenAIModel::Gpto3Mini => None,
            OpenAIModel::Gpt4120250414 => None,
            OpenAIModel::Gpt41Mini20250414 => None,
            OpenAIModel::Gpt41Nano20250414 => None,
            OpenAIModel::GptO4Mini20250416 => Some(ReasoningEffort::High),
            OpenAIModel::GptO320250416 => Some(ReasoningEffort::High),
            OpenAIModel::Sora10 => None,
        }
    }

    /// Returns true if this model is a video generation model (Sora)
    pub fn is_video_model(&self) -> bool {
        matches!(self, OpenAIModel::Sora10)
    }

    /// Returns the required video generation parameters for Sora models
    /// Returns None if this is not a video model
    pub fn video_params_info(&self) -> Option<SoraVideoParams> {
        match self {
            OpenAIModel::Sora10 => Some(SoraVideoParams {
                supported_sizes: vec!["1280x720", "720x1280"],
                supported_durations: vec!["4", "8", "12"],
                default_duration: "4",
            }),
            _ => None,
        }
    }

    /// Creates a Sora video request from this model if it's a video model
    /// Returns an error if this is not a video model or if parameters are invalid
    pub fn create_sora_request(
        &self,
        prompt: String,
        size: String,
        seconds: Option<String>,
    ) -> Result<crate::vendors::openai::sora_video_request::SoraVideoRequest, String> {
        if !self.is_video_model() {
            return Err(format!("Model {:?} is not a video generation model", self));
        }

        let model_id = match self {
            OpenAIModel::Sora10 => "sora-1.0",
            _ => return Err("Invalid video model".to_string()),
        };

        let request = crate::vendors::openai::sora_video_request::SoraVideoRequest::new(
            prompt,
            model_id.to_string(),
            size,
            seconds,
        );

        request.validate()?;
        Ok(request)
    }
}

/// Video generation parameters for Sora models
#[derive(Debug, Clone)]
pub struct SoraVideoParams {
    pub supported_sizes: Vec<&'static str>,
    pub supported_durations: Vec<&'static str>,
    pub default_duration: &'static str,
}
