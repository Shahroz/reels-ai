//! Specifies the desired format for the OpenAI model's response.
//!
//! Contains a `typ` field indicating the response format type (Text or JSON).
//! Used in the OpenAIChatCompletionRequest to control output structure.
//! Necessary for enabling JSON mode in compatible models.
//! Corresponds to OpenAI's `response_format` parameter object.

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")] // OpenAI uses 'type' field name
    pub typ: crate::vendors::openai::response_type::ResponseType, // FQN for ResponseType
}
