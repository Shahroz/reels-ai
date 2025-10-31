//! Represents the output from a Gemini model generation.
//!
//! This can either be a text response or a suggested function call.
//! Adheres to one-item-per-file and fully-qualified-path guidelines.

#[derive(Debug, Clone, PartialEq)]
pub enum GeminiOutput {
    Text(std::string::String),
    FunctionCall(crate::vendors::gemini::function_call_response::FunctionCallResponse),
    Mixed{text: String, function_calls: Vec<crate::vendors::gemini::function_call_response::FunctionCallResponse>},
    Image(crate::vendors::gemini::inline_data::InlineData),
}