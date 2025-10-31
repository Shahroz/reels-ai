//! Defines the structure for representing a tool that an agent can use.
//!
//! This includes the tool's name, a description of its capabilities,
//! and a JSON schema representation of its parameters (using `serde_json::Value`).
//! The struct derives common traits for serialization, debugging, and schema generation.

use serde_json::Value;

#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    utoipa::ToSchema
)]
pub struct ToolDefinition {
    pub name: std::string::String,
    pub description: std::string::String,
    pub parameters_json_schema: Value,
}
