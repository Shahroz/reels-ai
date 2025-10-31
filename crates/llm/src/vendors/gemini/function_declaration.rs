//! Defines the structure for a function declaration in the Gemini API.
//!
//! This represents a single function that the model can be instructed to call.
//! It includes the function's name, description, and a schema for its parameters.
//! This is a key component for enabling tool use with the Gemini models.
//! Adheres to the 'one item per file' rule and uses fully qualified paths where necessary.

// Re-export schema and property types for convenience in tests
pub use crate::vendors::gemini::function_parameters_schema::FunctionParametersSchema;
pub use crate::vendors::gemini::property_definition::PropertyDefinition;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    /// JSON schema for the function parameters.
    pub parameters: serde_json::Value,
}
