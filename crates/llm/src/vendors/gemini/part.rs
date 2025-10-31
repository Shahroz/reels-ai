//! Represents a single part of content within a Gemini API request, typically text.
//!
//! This struct can hold text data, inline data (e.g., for images), or both.
//! Multiple `Part` instances can exist within a single `Content` instance.
//! Uses fully qualified paths for dependencies.
//! Used within the `Content` structure.

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct Part {
    /// Optional text content for this part.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Optional inline data for this part (e.g., for images).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_data: Option<crate::vendors::gemini::inline_data::InlineData>,
    /// Optional file data for this part (e.g., for videos uploaded via File API).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<crate::vendors::gemini::file_data::FileData>,
    /// Optional field to carry the result of a function call.
    /// Used when sending a "function" role message back to the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_response: Option<crate::vendors::gemini::function_result_part::FunctionResultPart>,
    /// Optional field to echo a function call requested by the model.
    /// Used when constructing conversation history where a model's turn was a function call.
    #[serde(skip_serializing_if = "Option::is_none", rename = "functionCall")]
    pub function_call: Option<crate::vendors::gemini::function_call_response::FunctionCallResponse>,
}