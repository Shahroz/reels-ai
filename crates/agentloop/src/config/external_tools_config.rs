//! Defines the structure for passing external tool configurations.
//!
//! This module provides a struct to encapsulate the definitions and handlers
//! for external tools that are injected into the AgentLoop application.
//! Adheres strictly to the project's Rust coding standards.

/// Configuration for external tools.
///
/// Contains a list of tool definitions and a map of their corresponding handlers.
#[derive(std::fmt::Debug, std::clone::Clone)]
pub struct ExternalToolsConfig {
    /// A vector of tool definitions for external tools.
    pub definitions: std::vec::Vec<crate::types::tool_definition::ToolDefinition>,
    /// A map where keys are tool names and values are their `ToolHandler` functions.
    pub handlers: std::collections::HashMap<std::string::String, crate::tools::tool_handler::ToolHandler>,
}

impl ExternalToolsConfig {
    /// Creates a new, empty `ExternalToolsConfig`.
    pub fn new() -> Self {
        Self {
            definitions: std::vec::Vec::new(),
            handlers: std::collections::HashMap::new(),
        }
    }
}

impl std::default::Default for ExternalToolsConfig {
    fn default() -> Self {
        Self::new()
    }
}