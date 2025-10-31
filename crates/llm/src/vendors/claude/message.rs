//! Defines the structure for a message in the Claude API conversation history.
//!
//! Each message has a role ("user" or "assistant") and content.
//! The content is represented as a vector of `ContentBlock` enums.
//! This allows for potential future expansion beyond simple text.
//! Used within the `ClaudeMessageRequest` struct.

/// Represents a message in the conversation history.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: std::string::String, // "user" or "assistant"
    pub content: std::vec::Vec<crate::vendors::claude::content_block::ContentBlock>,
}

// Basic struct definition, tests might be more relevant in request/response serialization.
