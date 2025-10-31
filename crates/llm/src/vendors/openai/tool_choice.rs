//! Specifies how the OpenAI model should use the provided tools.
//!
//! Can be 'none', 'auto', or an object forcing a specific tool (function) call.
//! Controls the model's behavior regarding function calling.
//! Used in OpenAIChatCompletionRequest.
//! Allows forcing or disabling tool use.

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)] // Allows matching string or object
pub enum ToolChoice {
    String(String), // e.g., "none", "auto"
    Object { 
        #[serde(rename = "type")] // OpenAI uses 'type'
        typ: crate::vendors::openai::tool_type::ToolType, // FQN for ToolType
        // Assuming the object forces a function call by name
        // This structure might need refinement based on exact API spec
        function: FunctionChoice, // Define FunctionChoice struct below
    },
}

// Helper struct for the Object variant
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FunctionChoice {
    pub name: String,
}

