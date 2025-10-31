//! Defines a single entry in the conversation stream between user, agent, and tools.
//!
//! This struct captures who sent a message, the message content, when it was sent,
//! and any tools the agent chose to use in response or generation of this entry.
//! It adheres to the one-item-per-file guideline.
//! Uses fully qualified paths for types as per project standards.

// Required imports for derives and types according to guidelines (no `use` for internal types).
// Note: `chrono` and `serde` are external crates, so `use` is acceptable if needed for traits/macros,
// but fully qualified paths are preferred for types like DateTime, Utc, Serialize, Deserialize.

// Import ToSchema for OpenAPI documentation generation
use utoipa::ToSchema;

/// A single entry in the conversation stream.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, ToSchema)]
pub struct ConversationEntry {
    /// Unique identifier for this entry.
    pub id: uuid::Uuid,
    /// Identifier of the parent entry in a threaded conversation, if any.
    pub parent_id: Option<uuid::Uuid>,
    /// Depth of the entry in the conversation tree.
    pub depth: u32,
    /// Who sent the message (User, Agent, or Tool).
    pub sender: crate::types::sender::Sender,
    /// The textual content of the message.
    pub message: String,
    /// Timestamp when the entry was created. Uses fully qualified path.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Tools selected by the agent relevant to this entry. Uses fully qualified path.
    pub tools: Vec<crate::types::tool_choice::ToolChoice>, // Assuming ToolChoice is defined
    /// Attachments associated with this entry.
    pub attachments: Vec<crate::types::attachment::Attachment>,
    /// The specific tool choice made by the agent, if applicable.
    pub tool_choice: Option<crate::types::tool_choice::ToolChoice>,
    /// The response from a tool, if this entry represents a tool's output.
    pub tool_response: Option<crate::types::tool_response::ToolResponse>,
}

// No tests are defined here as per the instruction focusing only on the struct.
// If tests were required, they would follow the guidelines:
// #[cfg(FALSE)]
// mod tests {
//     #[test]
//     fn test_conversation_entry_creation() {
//         // ... test implementation using super::ConversationEntry ...
//         // and fully qualified paths for other types like std::assert_eq!
//     }
// }