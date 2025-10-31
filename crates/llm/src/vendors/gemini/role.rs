//! Defines the role of the entity providing the content in the Gemini API.
//!
//! This enum specifies whether the content originates from the "user" or the "model".
//! It is used within the `Content` struct to attribute parts of a conversation.
//! Adheres to one item per file and fully qualified path guidelines.
//! Ensures proper serialization for API communication.

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")] // Gemini API expects "user", "model"
pub enum Role {
    User,
    Model,
    Function,
}

#[cfg(test)]
mod tests {
    // Access the enum under test via `super::`. Full paths for other items.
    #[test]
    fn test_role_serialization_user() {
        let role = super::Role::User;
        // unwrap() is acceptable in tests.
        let serialized = serde_json::to_string(&role).unwrap();
        assert_eq!(serialized, "\"user\"");
    }

    #[test]
    fn test_role_serialization_model() {
        let role = super::Role::Model;
        let serialized = serde_json::to_string(&role).unwrap();
        assert_eq!(serialized, "\"model\"");
    }

    #[test]
    fn test_role_deserialization_user() {
        let json_input = "\"user\"";
        let deserialized: super::Role = serde_json::from_str(json_input).unwrap();
        assert_eq!(deserialized, super::Role::User);
    }

    #[test]
    fn test_role_deserialization_model() {
        let json_input = "\"model\"";
        let deserialized: super::Role = serde_json::from_str(json_input).unwrap();
        assert_eq!(deserialized, super::Role::Model);
    }

    #[test]
    fn test_role_serialization_function() {
        let role = super::Role::Function;
        let serialized = serde_json::to_string(&role).unwrap();
        assert_eq!(serialized, "\"function\"");
    }
}
