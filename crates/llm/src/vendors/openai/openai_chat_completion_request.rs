//! Defines the request structure for OpenAI's chat completion API.
//!
//! Includes all configurable parameters like messages, model, temperature,
//! tools, response format, reasoning, etc. Uses skip_serializing_none and serde_as.
//! Provides a constructor for simple user messages.
//! Represents the full request body sent to the API.

// Fully qualified paths needed for types from other modules/crates:
// HashMap -> std::collections::HashMap
// Message -> crate::vendors::openai::message::Message
// OpenAIModel -> crate::vendors::openai::openai_model::OpenAIModel
// ResponseFormat -> crate::vendors::openai::response_format::ResponseFormat
// StringOrArray -> crate::vendors::openai::string_or_array::StringOrArray (if stop uses it, original used Option<String>)
// Tool -> crate::vendors::openai::tool::Tool
// ToolChoice -> crate::vendors::openai::tool_choice::ToolChoice
// Reasoning -> crate::vendors::openai::reasoning::Reasoning
// Role -> crate::vendors::openai::role::Role
// serde_as -> serde_with::serde_as
// skip_serializing_none -> serde_with::skip_serializing_none

#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)] // Added Default derive
pub struct OpenAIChatCompletionRequest {
    pub messages: Vec<crate::vendors::openai::message::Message>,
    pub model: crate::vendors::openai::openai_model::OpenAIModel,
    pub frequency_penalty: Option<f32>,
    #[serde_as(as = "Option<Vec<(_, _)>>")]
    pub logit_bias: Option<std::collections::HashMap<i32, f32>>,
    pub max_tokens: Option<usize>,
    pub n: Option<usize>,
    pub response_format: Option<crate::vendors::openai::response_format::ResponseFormat>,
    pub seed: Option<usize>,
    pub stop: Option<String>, // Kept as Option<String> based on original field type
    // If StringOrArray was intended: pub stop: Option<crate::vendors::openai::string_or_array::StringOrArray>,
    pub stream: Option<bool>,
    pub temperature: Option<f32>,
    pub top_p: Option<usize>,
    pub tools: Option<Vec<crate::vendors::openai::tool::Tool>>,
    pub tool_choice: Option<crate::vendors::openai::tool_choice::ToolChoice>,
    pub user: Option<String>,
    // Updated field to match the API structure: reasoning: {effort: ...}
    pub reasoning: Option<crate::vendors::openai::reasoning::Reasoning>,
}

impl OpenAIChatCompletionRequest {
    //! Creates a new OpenAIChatCompletionRequest from a single user message.
    //!
    //! Initializes the request with the provided message (as User role)
    //! and the specified model. Sets reasoning effort based on the model.
    //! Fills other fields with defaults.
    //! Convenience constructor for basic use cases.
    pub fn new_from_user_message(
        message: std::string::String,
        model: crate::vendors::openai::openai_model::OpenAIModel,
    ) -> OpenAIChatCompletionRequest {
        OpenAIChatCompletionRequest {
            messages: std::vec![crate::vendors::openai::message::Message {
                content: Some(message),
                role: crate::vendors::openai::role::Role::User,
                name: None,
            }],
            model: model.clone(), // Clone model as it's moved otherwise
            // Construct the Reasoning struct if the model specifies an effort
            reasoning: model.reasoning_effort().map(|effort| crate::vendors::openai::reasoning::Reasoning { effort }),
            ..Default::default() // Use struct update syntax with Default
        }
    }
}
