//! Defines the structure for content blocks within a Claude API message.
//!
//! This enum represents different types of content that can be part of a message.
//! Currently, only supports text content.
//! Designed to be extensible for future content types (e.g., images).
//! Used within the `Message` struct.

/// Represents a block of content within a message.
/// Currently only supports text, but could be extended for images etc.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: std::string::String },
    // Add other content block types like "image" if needed
}

// Basic enum definition, tests might be more relevant in request/response serialization.
