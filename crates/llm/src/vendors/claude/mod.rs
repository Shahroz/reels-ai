//! Defines and re-exports items related to the Anthropic Claude API integration.
//!
//! This module organizes the various components (structs, enums, functions)
//! required for interacting with the Claude API, adhering to the one-item-per-file rule.
//! It re-exports the public items for convenient access from parent modules.
//! Follows the project's Rust coding standards.

// Declare modules for each item/file
pub mod call_claude_api;
pub mod claude_api_base_url;
pub mod claude_api_version;
pub mod claude_message_request;
pub mod claude_message_response;
pub mod claude_model;
pub mod content_block;
pub mod message;
pub mod usage;

// Re-export public items for external use
pub use call_claude_api::call_claude_api;
pub use claude_api_base_url::CLAUDE_API_BASE_URL;
pub use claude_api_version::CLAUDE_API_VERSION;
pub use claude_message_request::ClaudeMessageRequest;
pub use claude_message_response::ClaudeMessageResponse;
pub use claude_model::ClaudeModel;
pub use content_block::ContentBlock;
pub use message::Message;
pub use usage::Usage;

// Note: The original tests resided in the single file. They have been moved
// into the respective files for the items they test (e.g., tests for ClaudeModel
// are now in claude_model.rs).
