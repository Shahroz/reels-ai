//! Defines a single unified message within a conversation.
//!
//! Each message consists of a role (who sent it) and content (what was said).
//! This structure uses the `UnifiedRole` and `UnifiedMessageContent` enums
//! to provide a standardized representation across different LLM vendors.
//! It aims to capture the essential elements of a conversational turn.

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnifiedMessage {
    pub role: crate::unified_conversation::unified_role::UnifiedRole,
    pub content: crate::unified_conversation::unified_message_content::UnifiedMessageContent,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_unified_message_serialization_deserialization() {
        let message = super::UnifiedMessage {
            role: crate::unified_conversation::unified_role::UnifiedRole::User,
            content: crate::unified_conversation::unified_message_content::UnifiedMessageContent::Text(
                std::string::String::from("What is the weather like?"),
            ),
        };

        let serialized_message = serde_json::to_string(&message).unwrap();
        // Expected: {"role":"user","content":{"Text":"What is the weather like?"}}
        let expected_json = "{\"role\":\"user\",\"content\":{\"Text\":\"What is the weather like?\"}}";
        assert_eq!(serialized_message, expected_json);

        let deserialized_message: super::UnifiedMessage = serde_json::from_str(&serialized_message).unwrap();
        assert_eq!(deserialized_message, message);
    }
}