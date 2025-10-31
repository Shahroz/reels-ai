//! Defines the specific Replicate models supported by the application.
//!
//! This enum lists the various models available through the Replicate API endpoint.
//! It provides a method to retrieve the correct prediction endpoint URL for each model.
//! Conforms to the one-item-per-file standard.
//! Follows guidelines: no `use` statements, file preamble docs.

//! Revision History
//! - 2025-04-15T15:27:38Z @AI: Initial creation during refactor.

/// Represents the supported Replicate models.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum ReplicateModel {
    DeepseekR1,
    IbmGranite328bInstruct,
    IbmGraniteVision32_2b,
    MetaLlama31405bInstruct,
    MetaLlama370bInstruct,
    MetaLlama38bInstruct,
    #[default]
    Mistral7bV01,
}

impl ReplicateModel {
    /// Returns the API endpoint URL for the given model.
    pub fn endpoint(&self) -> &'static str {
        match self {
            ReplicateModel::DeepseekR1 => "https://api.replicate.com/v1/models/deepseek-ai/deepseek-r1/predictions",
            ReplicateModel::IbmGranite328bInstruct => "https://api.replicate.com/v1/models/ibm-granite/granite-3.2-8b-instruct/predictions",
            ReplicateModel::IbmGraniteVision32_2b => "https://api.replicate.com/v1/models/ibm-granite/granite-vision-3.2-2b/predictions",
            ReplicateModel::MetaLlama31405bInstruct => "https://api.replicate.com/v1/models/meta/meta-llama-3.1-405b-instruct/predictions",
            ReplicateModel::MetaLlama370bInstruct => "https://api.replicate.com/v1/models/meta/meta-llama-3-70b-instruct/predictions",
            ReplicateModel::MetaLlama38bInstruct => "https://api.replicate.com/v1/models/meta/meta-llama-3-8b-instruct/predictions",
            ReplicateModel::Mistral7bV01 => "https://api.replicate.com/v1/models/mistralai/mistral-7b-v0.1/predictions",
        }
    }
}

// No tests needed specifically for this enum/impl block as it's data/configuration.
