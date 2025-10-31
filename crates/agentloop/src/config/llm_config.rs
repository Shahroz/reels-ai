//! Defines the configuration structure for Language Model interactions.
//!
//! This file contains the `LlmConfig` struct, which holds settings
//! related to the LLM provider, model selection, API keys (if applicable),
//! and other relevant parameters for interacting with the language model.
//! Adheres to the one-item-per-file guideline.

// Note: `llm` crate path is specified in the workspace Cargo.toml or agentloop Cargo.toml.
// Full paths like `llm::llm_typed_unified::VendorModel` are used as per guidelines.

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub model: String, // Generic model identifier, may be less used with specific pools
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,

    // Model pools for specific tasks
   pub compaction_models: Vec<llm::llm_typed_unified::vendor_model::VendorModel>,
   pub context_termination_models: Vec<llm::llm_typed_unified::vendor_model::VendorModel>,
   pub tool_logic_models: Vec<llm::llm_typed_unified::vendor_model::VendorModel>,
   pub conversation_models: Vec<llm::llm_typed_unified::vendor_model::VendorModel>,
   pub check_termination_models: Vec<llm::llm_typed_unified::vendor_model::VendorModel>,
   pub context_evaluation_models: Vec<llm::llm_typed_unified::vendor_model::VendorModel>, // Added 2025-04-24
   pub summarization_models: Vec<llm::llm_typed_unified::vendor_model::VendorModel>,      // Added 2025-04-24
}

impl std::default::Default for LlmConfig {
    fn default() -> Self {
        LlmConfig {
            model: String::new(), // Default empty string for generic model
            api_key: None,
            temperature: None,
            max_tokens: None,
            // Default models using fully qualified paths
           compaction_models: vec![llm::llm_typed_unified::vendor_model::VendorModel::Gemini(
                llm::vendors::gemini::gemini_model::GeminiModel::Gemini20FlashLite,
            )],
           check_termination_models: vec![llm::llm_typed_unified::vendor_model::VendorModel::Gemini(
                llm::vendors::gemini::gemini_model::GeminiModel::Gemini20FlashLite, // Use a fast model for termination checks
            )],
           context_termination_models: vec![llm::llm_typed_unified::vendor_model::VendorModel::Gemini(
                llm::vendors::gemini::gemini_model::GeminiModel::Gemini20FlashLite,
            )],
           tool_logic_models: vec![llm::llm_typed_unified::vendor_model::VendorModel::Gemini(
                llm::vendors::gemini::gemini_model::GeminiModel::Gemini25Pro
            )],
           conversation_models: vec![llm::llm_typed_unified::vendor_model::VendorModel::Gemini(
               llm::vendors::gemini::gemini_model::GeminiModel::Gemini25Pro,
            )],
            // Added 2025-04-24
           context_evaluation_models: vec![llm::llm_typed_unified::vendor_model::VendorModel::Gemini(
                llm::vendors::gemini::gemini_model::GeminiModel::Gemini20FlashLite,
            )],
            // Added 2025-04-24
           summarization_models: vec![llm::llm_typed_unified::vendor_model::VendorModel::Gemini(
                llm::vendors::gemini::gemini_model::GeminiModel::Gemini20FlashLite,
            )],
        }
    }
}
