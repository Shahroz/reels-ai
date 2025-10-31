//! Defines the structure for a tool's response, covering success or failure.
//!
//! This enum encapsulates the outcome of a tool execution, providing a structured
//! way to represent either a successful result or an error.
//! Adheres to the one-item-per-file guideline and uses fully qualified paths.

// Import ToSchema for OpenAPI documentation generation, if this type is part of the API.
// use utoipa::ToSchema; // Uncomment if utoipa is used and this should be in schema.

use utoipa::ToSchema;

/// Represents the outcome of a tool execution.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)] // Add ToSchema if utoipa is used: #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub enum ToolResponse {
    /// Indicates a successful tool execution, containing the full response.
    Success(crate::types::full_tool_response::FullToolResponse),
    /// Indicates a failed tool execution, containing details about the failure.
    Failure(crate::types::user_tool_failure::UserToolFailure),
}

// No tests are defined here as this file primarily defines a data structure.
// If tests were required for logic within this enum (e.g., helper methods),
// they would follow the standard #[cfg(test)] mod tests { ... } structure.