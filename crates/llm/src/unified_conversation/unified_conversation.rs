//! Defines a unified structure for representing a conversation.
//!
//! A conversation is a sequence of messages exchanged between different roles
//! (e.g., user, assistant, system). This struct holds a vector of `UnifiedMessage`
//! instances, providing a common format for conversational data across LLM vendors.
//! It is designed to be easily serializable and deserializable.

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnifiedConversation {
    pub messages: std::vec::Vec<crate::unified_conversation::unified_message::UnifiedMessage>,
}

impl UnifiedConversation {
    /// Creates a new, empty conversation.
    pub fn new() -> Self {
        Self {
            messages: std::vec::Vec::new(),
        }
    }

    /// Adds a text message to the conversation.
    pub fn add_message(
        &mut self,
        role: crate::unified_conversation::unified_role::UnifiedRole,
        content_text: std::string::String,
    ) {
        self.messages.push(
            crate::unified_conversation::unified_message::UnifiedMessage {
                role,
                content: crate::unified_conversation::unified_message_content::UnifiedMessageContent::Text(content_text),
            }
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_unified_conversation_serialization_deserialization() {
        let mut conversation = super::UnifiedConversation::new();
        conversation.add_message(
            crate::unified_conversation::unified_role::UnifiedRole::User,
            std::string::String::from("Hello"),
        );
        conversation.add_message(
            crate::unified_conversation::unified_role::UnifiedRole::Assistant,
            std::string::String::from("Hi there!"),
        );

        let serialized_conversation = serde_json::to_string(&conversation).unwrap();
        let expected_json = "{\"messages\":[{\"role\":\"user\",\"content\":{\"Text\":\"Hello\"}},{\"role\":\"assistant\",\"content\":{\"Text\":\"Hi there!\"}}]}";
        assert_eq!(serialized_conversation, expected_json);

        let deserialized_conversation: super::UnifiedConversation = serde_json::from_str(&serialized_conversation).unwrap();
        assert_eq!(deserialized_conversation, conversation);
    }

    #[test]
    fn test_empty_conversation() {
        let conversation = super::UnifiedConversation::new();
        let serialized_conversation = serde_json::to_string(&conversation).unwrap();
        assert_eq!(serialized_conversation, "{\"messages\":[]}");
        let deserialized_conversation: super::UnifiedConversation = serde_json::from_str(&serialized_conversation).unwrap();
        assert_eq!(deserialized_conversation, conversation);
    }

    #[test]
    fn test_add_message() {
        let mut conversation = super::UnifiedConversation::new();
        conversation.add_message(
            crate::unified_conversation::unified_role::UnifiedRole::System,
            std::string::String::from("System prompt."),
        );
        assert_eq!(conversation.messages.len(), 1);
        assert_eq!(conversation.messages[0].role, crate::unified_conversation::unified_role::UnifiedRole::System);
        match &conversation.messages[0].content {
            crate::unified_conversation::unified_message_content::UnifiedMessageContent::Text(text) => {
                assert_eq!(text, "System prompt.");
            }
            // _ => panic!("Unexpected content type"), // Not needed if only Text is possible
        }
    }
}