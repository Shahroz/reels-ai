//! Defines the unified roles for messages in a conversation.
//!
//! This enum standardizes the possible senders of a message across different LLM vendors.
//! It includes System, User, and Assistant roles to cover common use cases.
//! Gemini's 'model' role maps to Assistant, and its system instructions can be represented as a System role message.
//! Serialization will use lowercase names for broader compatibility.

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnifiedRole {
    System,
    User,
    Assistant,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_unified_role_serialization_deserialization() {
        let role_user = super::UnifiedRole::User;
        let serialized_user = serde_json::to_string(&role_user).unwrap();
        assert_eq!(serialized_user, "\"user\"");
        let deserialized_user: super::UnifiedRole = serde_json::from_str(&serialized_user).unwrap();
        assert_eq!(deserialized_user, super::UnifiedRole::User);

        let role_system = super::UnifiedRole::System;
        let serialized_system = serde_json::to_string(&role_system).unwrap();
        assert_eq!(serialized_system, "\"system\"");
        let deserialized_system: super::UnifiedRole = serde_json::from_str(&serialized_system).unwrap();
        assert_eq!(deserialized_system, super::UnifiedRole::System);

        let role_assistant = super::UnifiedRole::Assistant;
        let serialized_assistant = serde_json::to_string(&role_assistant).unwrap();
        assert_eq!(serialized_assistant, "\"assistant\"");
        let deserialized_assistant: super::UnifiedRole = serde_json::from_str(&serialized_assistant).unwrap();
        assert_eq!(deserialized_assistant, super::UnifiedRole::Assistant);
    }
}