//! Database model for logo collections.
//!
//! This module defines the LogoCollection struct representing a collection of logo assets.
//! Logo collections allow users to organize their logo assets for watermarking operations.
//! Each collection has a name, optional description, and belongs to a specific user.

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct LogoCollection {
    #[schema(format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    #[schema(format = "uuid", value_type = String)]
    pub user_id: uuid::Uuid,
    pub name: std::string::String,
    pub description: std::option::Option<std::string::String>,
    #[schema(format = "date-time", value_type = String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(format = "date-time", value_type = String)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_logo_collection_serialization() {
        let logo_collection = LogoCollection {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            name: "Test Collection".to_string(),
            description: std::option::Option::Some("Test description".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Test serialization
        let serialized = serde_json::to_string(&logo_collection).unwrap();
        assert!(serialized.contains("Test Collection"));
        assert!(serialized.contains("Test description"));

        // Test deserialization
        let deserialized: LogoCollection = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.name, logo_collection.name);
        assert_eq!(deserialized.description, logo_collection.description);
        assert_eq!(deserialized.id, logo_collection.id);
        assert_eq!(deserialized.user_id, logo_collection.user_id);
    }

    #[test]
    fn test_logo_collection_serialization_without_description() {
        let logo_collection = LogoCollection {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            name: "Test Collection".to_string(),
            description: std::option::Option::None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Test serialization with null description
        let serialized = serde_json::to_string(&logo_collection).unwrap();
        assert!(serialized.contains("Test Collection"));
        assert!(serialized.contains("null") || serialized.contains("\"description\":null"));

        // Test deserialization
        let deserialized: LogoCollection = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.name, logo_collection.name);
        assert_eq!(deserialized.description, std::option::Option::None);
    }

    #[test]
    fn test_logo_collection_json_edge_cases() {
        // Test with special characters in name and description
        let logo_collection = LogoCollection {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            name: "Collection with \"quotes\" & <tags>".to_string(),
            description: std::option::Option::Some("Description with émojis 🎨 and newlines\ntest".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&logo_collection).unwrap();
        let deserialized: LogoCollection = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.name, logo_collection.name);
        assert_eq!(deserialized.description, logo_collection.description);
    }

    #[test]
    fn test_logo_collection_clone() {
        let logo_collection = LogoCollection {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            name: "Test Collection".to_string(),
            description: std::option::Option::Some("Test description".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let cloned = logo_collection.clone();
        assert_eq!(cloned.id, logo_collection.id);
        assert_eq!(cloned.name, logo_collection.name);
        assert_eq!(cloned.description, logo_collection.description);
    }

    #[test]
    fn test_logo_collection_debug() {
        let logo_collection = LogoCollection {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            name: "Test Collection".to_string(),
            description: std::option::Option::Some("Test description".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let debug_str = std::format!("{:?}", logo_collection);
        assert!(debug_str.contains("LogoCollection"));
        assert!(debug_str.contains("Test Collection"));
    }
}
