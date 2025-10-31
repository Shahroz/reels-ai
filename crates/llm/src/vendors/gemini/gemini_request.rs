//! Represents the main request body sent to the Gemini API generateContent endpoint.
//!
//! Contains the prompt content, generation configuration, safety settings,
//! tools configuration, and optionally, system instructions to guide the model.
//! This structure encapsulates all necessary parameters for a generation request.
//! Uses fully qualified paths for all types.

#[derive(Debug, serde::Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<crate::vendors::gemini::content::Content>,
   #[serde(rename = "generationConfig")]
   pub generation_config: crate::vendors::gemini::generation_config::GenerationConfig,
   // #[serde(rename = "safetySettings")]
  // pub safety_settings: Vec<crate::vendors::gemini::safety_setting::SafetySetting>,
   #[serde(skip_serializing_if = "Option::is_none")]
   pub tools: Option<Vec<crate::vendors::gemini::tool::Tool>>, // Defines tools the model can use, typically function declarations.
   #[serde(rename = "system_instruction", skip_serializing_if = "Option::is_none")]
   pub system_instruction: Option<crate::vendors::gemini::system_instruction::SystemInstruction>,
}
