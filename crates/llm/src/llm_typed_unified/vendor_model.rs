//! Defines the `VendorModel` enum and its associated implementations.
//!
//! This enum represents the different LLM vendors and their specific models
//! supported by the system. It includes implementations for display, default
//! value, and obtaining model aliases.
//! Adheres to the "one item per file" and fully qualified path guidelines.

use std::fmt;
use strum_macros::EnumIter;
use serde::{Serialize, Deserialize};

// Import specific model types using their fully qualified paths to the defining module
use crate::vendors::claude::claude_model::ClaudeModel;
use crate::vendors::gemini::gemini_model::GeminiModel;
use crate::vendors::openai::openai_model::OpenAIModel;
use crate::vendors::replicate::replicate_model::ReplicateModel;

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum VendorModel {
    OpenAI(OpenAIModel),
    Gemini(GeminiModel),
    Claude(ClaudeModel),
    Replicate(ReplicateModel),
}

impl fmt::Display for VendorModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use Debug formatting as a simple way to get a string representation
        // write! macro is in prelude.
        write!(f, "{:?}", self)
    }
}

impl Default for VendorModel {
    fn default() -> Self {
        // OpenAIModel::Gpto3Mini is accessed via the imported OpenAIModel type.
        VendorModel::OpenAI(OpenAIModel::Gpto3Mini)
    }
}

impl VendorModel {
    // Vec is a prelude type. &'static str is fundamental.
    pub fn aliases(&self) -> Vec<&'static str> {
        match self {
            // model.aliases() calls are on the imported model types.
            VendorModel::OpenAI(model) => model.aliases(),
            VendorModel::Gemini(model) => model.aliases(),
            VendorModel::Claude(model) => {
                match model {
                    // Enum variants are accessed via the imported ClaudeModel type.
                    ClaudeModel::Claude35SonnetLatest => vec![],
                    ClaudeModel::Claude3Opus => vec![],
                    ClaudeModel::Claude3Sonnet => vec![],
                    ClaudeModel::Claude3Haiku => vec![],
                    ClaudeModel::Claude37SonnetLatest => vec!["claude"], // String literals are fine.
                }
            }
            VendorModel::Replicate(model) => {
                match model {
                    // Enum variants are accessed via the imported ReplicateModel type.
                    ReplicateModel::DeepseekR1 => vec!["deepseek-r1"],
                    ReplicateModel::IbmGranite328bInstruct => vec!["granite-8b-instruct"],
                    ReplicateModel::IbmGraniteVision32_2b => vec!["granite-vision-2b"],
                    ReplicateModel::MetaLlama31405bInstruct => vec!["llama3.1-405b-instruct"],
                    ReplicateModel::MetaLlama370bInstruct => vec!["llama3-70b-instruct"],
                    ReplicateModel::MetaLlama38bInstruct => vec!["llama3-8b-instruct"],
                    ReplicateModel::Mistral7bV01 => vec!["mistral-7b-v0.1"],
                }
            }
        }
    }

    /// Returns true if this model is a video generation model (Sora or Veo3)
    pub fn is_video_model(&self) -> bool {
        match self {
            VendorModel::OpenAI(model) => model.is_video_model(),
            VendorModel::Gemini(model) => model.is_video_model(),
            VendorModel::Claude(_) => false,
            VendorModel::Replicate(_) => false,
        }
    }

    /// Returns the model identifier string (e.g., "sora-1.0", "veo-3", "gpt-4o")
    /// Uses the serde rename attribute for OpenAI models, and id() method for Gemini models
    pub fn model_id(&self) -> String {
        match self {
            VendorModel::OpenAI(model) => {
                // Use serde_json::to_string which respects #[serde(rename = "...")] attributes
                serde_json::to_string(model)
                    .unwrap_or_else(|_| format!("{:?}", model))
                    .trim_matches('"')
                    .to_string()
            }
            VendorModel::Gemini(model) => model.id().to_string(),
            VendorModel::Claude(model) => {
                serde_json::to_string(model)
                    .unwrap_or_else(|_| format!("{:?}", model))
                    .trim_matches('"')
                    .to_string()
            }
            VendorModel::Replicate(model) => {
                serde_json::to_string(model)
                    .unwrap_or_else(|_| format!("{:?}", model))
                    .trim_matches('"')
                    .to_string()
            }
        }
    }
}