//! Module for agent tool dispatching, parameter definitions, and handlers.
//!
//! This module organizes the components related to handling agent tool
//! requests within the Narrativ backend. It adheres to the project's
//! one-item-per-file coding standards. Each submodule typically contains
//! a single primary Rust item or further organizes related items.

pub mod dispatch_narrativ_agent_tool;
pub mod gemini_tool_conversion;
pub mod handlers; // Contains individual tool handler functions/modules
pub mod narrativ_tool_parameters; // Defines the NarrativToolParameters enum
pub mod tool_params; // Contains specific parameter structs for each tool

// No re-exports are made here to enforce usage of fully qualified paths
// as per project guidelines, e.g., crate::agent_tools::dispatch_narrativ_agent_tool::...
