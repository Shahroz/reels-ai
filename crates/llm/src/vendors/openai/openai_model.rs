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
            OpenAIModel::GptO320250416 => vec!["o3"]
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
        }
    }
}
