//! Database model for logo collection assets.
//!
//! This module defines the LogoCollectionAsset struct representing the junction table
//! between logo collections and assets. It associates specific assets with logo collections
//! and includes optional display names for logos within collections.

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct LogoCollectionAsset {
    #[schema(format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    #[schema(format = "uuid", value_type = String)]
    pub logo_collection_id: uuid::Uuid,
    #[schema(format = "uuid", value_type = String)]
    pub asset_id: uuid::Uuid,
    pub display_name: std::option::Option<std::string::String>,
    #[schema(format = "date-time", value_type = String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_logo_collection_asset_serialization() {
        let asset = LogoCollectionAsset {
            id: uuid::Uuid::new_v4(),
            logo_collection_id: uuid::Uuid::new_v4(),
            asset_id: uuid::Uuid::new_v4(),
            display_name: std::option::Option::Some("Primary Logo".to_string()),
            created_at: chrono::Utc::now(),
        };

        // Test serialization
        let serialized = serde_json::to_string(&asset).unwrap();
        assert!(serialized.contains("Primary Logo"));

        // Test deserialization
        let deserialized: LogoCollectionAsset = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, asset.id);
        assert_eq!(deserialized.logo_collection_id, asset.logo_collection_id);
        assert_eq!(deserialized.asset_id, asset.asset_id);
        assert_eq!(deserialized.display_name, asset.display_name);
    }

    #[test]
    fn test_logo_collection_asset_without_display_name() {
        let asset = LogoCollectionAsset {
            id: uuid::Uuid::new_v4(),
            logo_collection_id: uuid::Uuid::new_v4(),
            asset_id: uuid::Uuid::new_v4(),
            display_name: std::option::Option::None,
            created_at: chrono::Utc::now(),
        };

        // Test serialization with null display_name
        let serialized = serde_json::to_string(&asset).unwrap();
        assert!(serialized.contains("null") || serialized.contains("\"display_name\":null"));

        // Test deserialization
        let deserialized: LogoCollectionAsset = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.display_name, std::option::Option::None);
        assert_eq!(deserialized.id, asset.id);
    }

    #[test]
    fn test_logo_collection_asset_with_special_characters() {
        let asset = LogoCollectionAsset {
            id: uuid::Uuid::new_v4(),
            logo_collection_id: uuid::Uuid::new_v4(),
            asset_id: uuid::Uuid::new_v4(),
            display_name: std::option::Option::Some("Logo with \"quotes\" & émojis 🎨".to_string()),
            created_at: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&asset).unwrap();
        let deserialized: LogoCollectionAsset = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.display_name, asset.display_name);
    }

    #[test]
    fn test_logo_collection_asset_clone() {
        let asset = LogoCollectionAsset {
            id: uuid::Uuid::new_v4(),
            logo_collection_id: uuid::Uuid::new_v4(),
            asset_id: uuid::Uuid::new_v4(),
            display_name: std::option::Option::Some("Test Logo".to_string()),
            created_at: chrono::Utc::now(),
        };

        let cloned = asset.clone();
        assert_eq!(cloned.id, asset.id);
        assert_eq!(cloned.logo_collection_id, asset.logo_collection_id);
        assert_eq!(cloned.asset_id, asset.asset_id);
        assert_eq!(cloned.display_name, asset.display_name);
    }

    #[test]
    fn test_logo_collection_asset_debug() {
        let asset = LogoCollectionAsset {
            id: uuid::Uuid::new_v4(),
            logo_collection_id: uuid::Uuid::new_v4(),
            asset_id: uuid::Uuid::new_v4(),
            display_name: std::option::Option::Some("Test Logo".to_string()),
            created_at: chrono::Utc::now(),
        };

        let debug_str = std::format!("{:?}", asset);
        assert!(debug_str.contains("LogoCollectionAsset"));
        assert!(debug_str.contains("Test Logo"));
    }
}
