//! Represents a tool, like a function, that the OpenAI model can call.
//!
//! Contains the type of the tool (e.g., Function) and its definition,
//! including the function name and parameters (schema often represented as JSON).
//! Used in OpenAIChatCompletionRequest to provide callable functions.
//! Enables function calling capabilities.

// Note: Parameters field was commented out in original, keeping it simple here.
// If parameters (JSON schema) are needed, add `pub parameters: serde_json::Value,`

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Tool {
    #[serde(rename = "type")] // OpenAI uses 'type'
    pub typ: crate::vendors::openai::tool_type::ToolType, // FQN for ToolType
    // Assuming the function definition itself is the primary content here
    // Based on common OpenAI usage, this often contains a 'function' object
    // For simplicity, let's assume it just needs the name for now as per original code
    // If a nested structure is needed, adjust this definition.
    // Example if it wraps a function definition object:
    // pub function: FunctionDefinition, // Where FunctionDefinition is another struct
    // For now, matching the simplified original structure:
    pub name: String, // Assuming 'name' refers to the function name within the tool definition
}
