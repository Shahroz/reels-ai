//! Defines a single message within the conversation history, potentially including attachments.
//!
//! This struct represents a message sent by either the user, the assistant (agent), or the system.
//! It includes the role, content, and any associated attachments.
//! Adheres to the one-item-per-file guideline and uses fully qualified paths.

// No revision history needed for initial implementation.

// (No `use crate::types::attachment::Attachment;` as FQN is used in struct definition)
use utoipa::ToSchema;
use schemars::JsonSchema;

/// Represents a message with a role (sender) and content.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ToSchema, JsonSchema)]
pub struct Message {
    /// The role indicating the sender (user, assistant, system).
    pub role: String,
    /// The textual content of the message.
    pub content: String,
    /// Optional attachments associated with the message.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: std::vec::Vec<crate::types::attachment::Attachment>,
}

impl Message {
    /// Creates a new message with the 'user' role.
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
            attachments: std::vec::Vec::new(),
        }
    }

    /// Creates a new message with the 'assistant' role.
    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            attachments: std::vec::Vec::new(),
        }
    }

    /// Creates a new message with the 'system' role.
    pub fn system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content,
            attachments: std::vec::Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    // Access the struct under test via `super::`. Full paths for other items.

    #[test]
    fn test_message_user_creation() {
        let msg = super::Message::user("Hello there".to_string());
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello there");
        assert!(msg.attachments.is_empty(), "Attachments should be empty for new user message");
    }

    #[test]
    fn test_message_assistant_creation() {
        let msg = super::Message::assistant("How can I help?".to_string());
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, "How can I help?");
        assert!(msg.attachments.is_empty(), "Attachments should be empty for new assistant message");
    }

    #[test]
    fn test_message_system_creation() {
        let msg = super::Message::system("Be concise.".to_string());
        assert_eq!(msg.role, "system");
        assert_eq!(msg.content, "Be concise.");
        assert!(msg.attachments.is_empty(), "Attachments should be empty for new system message");
    }

    #[test]
    fn test_message_equality() {
        let msg1 = super::Message::user("Test".to_string());
        let msg2 = super::Message::user("Test".to_string());
        let msg3 = super::Message::user("Different".to_string());
        let msg4 = super::Message::assistant("Test".to_string());

        assert_eq!(msg1, msg2);
        assert_ne!(msg1, msg3);
        assert_ne!(msg1, msg4);
    }

    #[test]
    fn test_message_clone() {
        let msg1 = super::Message::system("System prompt".to_string());
        let msg2 = msg1.clone();
        assert_eq!(msg1, msg2);
        // Ensure it's a true clone (modify one doesn't affect other - though Strings clone deeply anyway)
        assert!(msg2.attachments.is_empty(), "Cloned message attachments should be empty");
    }

    #[test]
    fn test_message_serialization_deserialization_without_attachments() {
        let msg = super::Message {
            role: "user".to_string(),
            content: "Test content".to_string(),
            attachments: std::vec::Vec::new(),
        };

        let serialized = serde_json::to_string(&msg).expect("Serialization failed");
        // Due to skip_serializing_if = "Vec::is_empty", "attachments" field should not be present
        assert!(!serialized.contains("attachments"), "Serialized JSON should not contain empty attachments field: {}", serialized);

        let deserialized: super::Message = serde_json::from_str(&serialized).expect("Deserialization failed");
        assert_eq!(deserialized.role, msg.role);
        assert_eq!(deserialized.content, msg.content);
        assert!(deserialized.attachments.is_empty(), "Deserialized attachments should be empty");
    }

    #[test]
    fn test_message_serialization_deserialization_with_attachments() {
        let text_content = std::string::String::from("Attachment content");
        let attachment1 = crate::types::attachment::Attachment {
            title: Some(std::string::String::from("Doc1")),
            kind: crate::types::attachment_type::AttachmentType::Text(
                crate::types::text_attachment::TextAttachment { content: text_content.clone() }
            ),
        };
        let msg = super::Message {
            role: "user".to_string(),
            content: "Test content with attachment".to_string(),
            attachments: std::vec![attachment1.clone()],
        };

        let serialized = serde_json::to_string(&msg).expect("Serialization failed");
        assert!(serialized.contains("attachments"), "Serialized JSON should contain attachments field: {}", serialized);
        assert!(serialized.contains("Doc1"), "Serialized JSON should contain attachment title: {}", serialized);

        let deserialized: super::Message = serde_json::from_str(&serialized).expect("Deserialization failed");
        assert_eq!(deserialized.role, msg.role);
        assert_eq!(deserialized.content, msg.content);
        assert_eq!(deserialized.attachments.len(), 1, "Deserialized attachments count mismatch");

        let deserialized_attachment = &deserialized.attachments[0];
        assert_eq!(deserialized_attachment.title, attachment1.title);

        match &deserialized_attachment.kind {
            crate::types::attachment_type::AttachmentType::Text(text_att) => {
                assert_eq!(text_att.content, text_content, "Deserialized attachment content mismatch");
            }
            _ => panic!("Unexpected attachment kind after deserialization"),
        }
    }
}