//! Aggregates and exposes the agent tool functionalities.
//!
//! This module declares sub-modules responsible for defining tool types,
//! individual tool handlers, registration, and dispatching logic, following
//! the project's "one item per file" structure.
//! Adheres strictly to the project's Rust coding standards.

pub mod tool_handler;
pub mod dispatch_tools;
pub mod tools_schema;

// Specific handler logic

// internal_params and internal_tool_wrappers might be obsolete or part of internal_tools_registry now.
// For this refactoring, ensure internal_tools_registry.rs is the source of truth for internal tool defs/handlers.
// The context file `internal_tools.rs` was modified to act as `internal_tools_registry.rs`.
// If `internal_tools.rs` was a separate entity, it might be removed or its contents merged.
// Assuming `internal_tools_registry` is the module name for the file previously named `internal_tools.rs` in context.

// Re-export key functions/types if needed for easier access from outside `tools`
// Example: pub use dispatch_tools::dispatch_tools;
// Example: pub use register_tools::register_tools;
// Example: pub use tool_handler::ToolHandler;
