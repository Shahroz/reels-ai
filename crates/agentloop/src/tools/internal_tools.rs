//! Provides the function to register all available agent tool handlers.
//!
//! THIS MODULE IS LARGELY DEPRECATED as tools are now provided by the host.
//! Functions here will return empty collections.
//! adapted to use strongly-typed parameters.
//! Adheres strictly to the project's Rust coding standards.
//! Follows the "one item per file" rule.

//! Revision History
//! - 2025-05-13T19:28:30Z @AI: Add get_internal_tool_definitions and rename register_tools.
//! - 2025-04-24T16:25:23Z @AI: Refactor wrappers to accept ToolParameters.
//! - 2025-04-24T14:52:38Z @AI: Fix E0308 by explicitly typing closure arguments.
//! - 2025-04-24T14:45:42Z @AI: Fix E0605 by boxing async handlers.

// Types needed for ToolHandler signature - Use FQN
// No longer need Arc/Mutex here as they are part of the signature types.

// Wrapper functions (search_tool_handler, browse_tool_handler, save_context_tool_handler)
// are removed as tool execution is now handled by host-provided handlers
// that already match the ToolHandler signature.

/// Retrieves definitions for all built-in internal tools.
///
/// # Returns
///
/// A `std::vec::Vec` of `crate::types::tool_definition::ToolDefinition` for internal tools.
pub fn get_internal_tool_definitions() -> std::vec::Vec<crate::types::tool_definition::ToolDefinition> {
    // Host now provides all tool definitions. Agentloop does not define any internally.
    std::vec::Vec::new()
}

/// Retrieves handlers for all built-in internal tools.
///
/// Returns an empty map as all tool handlers are now provided by the host.
///
/// # Returns
///
/// A `std::collections::HashMap` mapping tool names (`String`) to their handler functions (`ToolHandler`).
pub fn get_internal_tool_handlers() -> std::collections::HashMap<String, crate::tools::tool_handler::ToolHandler> {
    // Host now provides all tool handlers. Agentloop does not define any internally.
    std::collections::HashMap::new()
}

#[cfg(test)] // Changed from FALSE
mod tests {
    #[test]
    fn test_register_tools_contains_expected_keys() {
        // Test that the registration map contains the keys for known tools.
        // Now expects an empty map.
        let handlers = super::get_internal_tool_handlers();
        std::assert!(handlers.is_empty(), "Internal tool handlers map should be empty.");
    }

    #[test]
    fn test_get_internal_tool_definitions_returns_expected_tools() {
        // Test that definitions are returned for known tools.
        // Now expects an empty vector.
        let definitions = super::get_internal_tool_definitions();
        std::assert!(definitions.is_empty(), "Internal tool definitions vector should be empty.");
    }
}
