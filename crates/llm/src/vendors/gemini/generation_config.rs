//! Defines the generation parameters for the Gemini API request.
//!
//! Specifies settings like temperature, maximum output tokens, top P, and response modalities.
//! Controls the behavior of the language model during generation.
//! Uses fully qualified paths for dependencies.
//! Part of the main `Request` structure.

#[derive(Debug, serde::Serialize)]
pub struct GenerationConfig {
    // #[serde(rename = "responseModalities")]
    // pub response_modalities: std::vec::Vec<std::string::String>,
    pub temperature: f64,
    #[serde(rename = "maxOutputTokens")]
    pub max_output_tokens: u32,
    #[serde(rename = "topP")]
    pub top_p: f64,
    pub seed: u64,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_output_tokens: 8192,
            top_p: 1.0,
            seed: 0,
        }
    }
}
