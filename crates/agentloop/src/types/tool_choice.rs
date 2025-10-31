//! Defines the structure for representing a tool choice made by the agent.
//!
//! This struct holds the name of the selected tool and the strongly-typed parameters
//! required for its execution, using the `ToolParameters` enum.
//! Adheres to one-item-per-file and FQN standards.
//! Ensures type safety and schema generation for tool calls.

/// Represents a tool selected for execution, with strongly-typed parameters.
#[derive(Debug, utoipa::ToSchema, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ToolChoice {
    /// The parameters required by the tool, represented as raw JSON. The response should have format {"
    pub parameters: serde_json::Value,
}

