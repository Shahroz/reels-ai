//! Defines the unified content of a message in a conversation.
//!
//! This enum allows for different types of content within a message,
//! starting with simple text. It can be expanded to include other types
//! like images or tool calls as needed for broader LLM API compatibility.
//! For now, it primarily supports textual content.

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UnifiedMessageContent {
    Text(std::string::String),
    // Potentially: ImageUrl(std::string::String),
    // Potentially: ToolCall(SomeToolCallStruct),
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_unified_message_content_text_serialization_deserialization() {
        let content = super::UnifiedMessageContent::Text(std::string::String::from("Hello, world!"));
        let serialized_content = serde_json::to_string(&content).unwrap();
        // Expected: {"Text":"Hello, world!"}
        assert_eq!(serialized_content, "{\"Text\":\"Hello, world!\"}");
        let deserialized_content: super::UnifiedMessageContent = serde_json::from_str(&serialized_content).unwrap();
        assert_eq!(deserialized_content, content);
    }
}