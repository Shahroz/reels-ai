//! Schema definitions for logo collection API requests and responses.
//!
//! This module defines the data transfer objects for logo collection operations.
//! It includes request schemas for creating and updating logo collections,
//! as well as response schemas that include associated asset information.
//! All schemas follow OpenAPI specification for automatic documentation.

/// Request schema for creating a new logo collection
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct CreateLogoCollectionRequest {
    pub name: std::string::String,
    pub description: std::option::Option<std::string::String>,
    pub asset_ids: std::option::Option<std::vec::Vec<uuid::Uuid>>,
}

/// Request schema for updating an existing logo collection
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct UpdateLogoCollectionRequest {
    pub name: std::option::Option<std::string::String>,
    pub description: std::option::Option<std::string::String>,
}

/// Request schema for adding an asset to a logo collection
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct AddAssetToCollectionRequest {
    #[schema(format = "uuid", value_type = String)]
    pub asset_id: uuid::Uuid,
    pub display_name: std::option::Option<std::string::String>,
}

/// Response schema for logo collection with associated assets
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct LogoCollectionResponse {
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
    pub assets: std::vec::Vec<LogoCollectionAssetResponse>,
}

/// Response schema for logo collection asset with asset details
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct LogoCollectionAssetResponse {
    #[schema(format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    #[schema(format = "uuid", value_type = String)]
    pub asset_id: uuid::Uuid,
    pub display_name: std::option::Option<std::string::String>,
    #[schema(format = "date-time", value_type = String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub asset_name: std::string::String,
    pub asset_url: std::string::String,
    pub asset_type: std::string::String,
}

/// Response schema for listing logo collections (summary view)
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct LogoCollectionSummaryResponse {
    #[schema(format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    pub name: std::string::String,
    pub description: std::option::Option<std::string::String>,
    #[schema(format = "date-time", value_type = String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(format = "date-time", value_type = String)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub asset_count: i64,
    pub thumbnail_url: std::option::Option<std::string::String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_create_logo_collection_request_serialization() {
        let request = CreateLogoCollectionRequest {
            name: "Brand Logos".to_string(),
            description: std::option::Option::Some("Company brand assets".to_string()),
            asset_ids: std::option::Option::Some(vec![uuid::Uuid::new_v4(), uuid::Uuid::new_v4()]),
        };

        // Test serialization
        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("Brand Logos"));
        assert!(serialized.contains("Company brand assets"));

        // Test deserialization
        let deserialized: CreateLogoCollectionRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.name, request.name);
        assert_eq!(deserialized.description, request.description);
        assert_eq!(deserialized.asset_ids.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_create_logo_collection_request_minimal() {
        let request = CreateLogoCollectionRequest {
            name: "Minimal Collection".to_string(),
            description: std::option::Option::None,
            asset_ids: std::option::Option::None,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: CreateLogoCollectionRequest = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.name, "Minimal Collection");
        assert!(deserialized.description.is_none());
        assert!(deserialized.asset_ids.is_none());
    }

    #[test]
    fn test_create_request_validation_edge_cases() {
        // Test empty name (should be handled by validation layer)
        let json = r#"{"name": ""}"#;
        let request: CreateLogoCollectionRequest = serde_json::from_str(json).unwrap();
        assert!(request.name.is_empty());
        
        // Test with asset IDs array
        let asset_id = uuid::Uuid::new_v4();
        let json = std::format!(r#"{{
            "name": "Test Collection",
            "asset_ids": ["{}"]
        }}"#, asset_id);
        let request: CreateLogoCollectionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request.asset_ids.unwrap()[0], asset_id);
    }

    #[test]
    fn test_create_request_with_special_characters() {
        let json = r#"{
            "name": "Collection with \"quotes\" & émojis 🎨",
            "description": "Description with newlines\nand special chars: <>&\"'"
        }"#;

        let request: CreateLogoCollectionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Collection with \"quotes\" & émojis 🎨");
        assert_eq!(request.description.unwrap(), "Description with newlines\nand special chars: <>&\"'");
    }

    #[test]
    fn test_update_request_partial_updates() {
        // Test that all fields are optional for updates
        let json = r#"{}"#;
        let request: UpdateLogoCollectionRequest = serde_json::from_str(json).unwrap();
        assert!(request.name.is_none());
        assert!(request.description.is_none());
        
        // Test partial update with just name
        let json = r#"{"name": "Updated Name"}"#;
        let request: UpdateLogoCollectionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name.unwrap(), "Updated Name");
        assert!(request.description.is_none());

        // Test partial update with both fields
        let json = r#"{"name": "New Name", "description": "New Description"}"#;
        let request: UpdateLogoCollectionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name.unwrap(), "New Name");
        assert_eq!(request.description.unwrap(), "New Description");

        // Test setting description to null
        let json = r#"{"description": null}"#;
        let request: UpdateLogoCollectionRequest = serde_json::from_str(json).unwrap();
        assert!(request.name.is_none());
        assert!(request.description.is_none());
    }

    #[test]
    fn test_add_asset_request_serialization() {
        let asset_id = uuid::Uuid::new_v4();
        let request = AddAssetToCollectionRequest {
            asset_id,
            display_name: std::option::Option::Some("Primary Logo".to_string()),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: AddAssetToCollectionRequest = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.asset_id, asset_id);
        assert_eq!(deserialized.display_name.unwrap(), "Primary Logo");
    }

    #[test]
    fn test_add_asset_request_without_display_name() {
        let asset_id = uuid::Uuid::new_v4();
        let json = std::format!(r#"{{
            "asset_id": "{}"
        }}"#, asset_id);

        let request: AddAssetToCollectionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request.asset_id, asset_id);
        assert!(request.display_name.is_none());
    }

    #[test]
    fn test_logo_collection_response_serialization() {
        let collection_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let asset_response = LogoCollectionAssetResponse {
            id: uuid::Uuid::new_v4(),
            asset_id: uuid::Uuid::new_v4(),
            display_name: std::option::Option::Some("Test Logo".to_string()),
            created_at: chrono::Utc::now(),
            asset_name: "logo.png".to_string(),
            asset_url: "https://example.com/logo.png".to_string(),
            asset_type: "image/png".to_string(),
        };

        let response = LogoCollectionResponse {
            id: collection_id,
            user_id,
            name: "Test Collection".to_string(),
            description: std::option::Option::Some("Test description".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            assets: vec![asset_response],
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("Test Collection"));
        assert!(serialized.contains("Test Logo"));
        assert!(serialized.contains("logo.png"));

        let deserialized: LogoCollectionResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, collection_id);
        assert_eq!(deserialized.name, "Test Collection");
        assert_eq!(deserialized.assets.len(), 1);
        assert_eq!(deserialized.assets[0].asset_name, "logo.png");
    }

    #[test]
    fn test_logo_collection_asset_response_serialization() {
        let asset_response = LogoCollectionAssetResponse {
            id: uuid::Uuid::new_v4(),
            asset_id: uuid::Uuid::new_v4(),
            display_name: std::option::Option::None,
            created_at: chrono::Utc::now(),
            asset_name: "logo.svg".to_string(),
            asset_url: "https://example.com/logo.svg".to_string(),
            asset_type: "image/svg+xml".to_string(),
        };

        let serialized = serde_json::to_string(&asset_response).unwrap();
        let deserialized: LogoCollectionAssetResponse = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.asset_name, "logo.svg");
        assert_eq!(deserialized.asset_type, "image/svg+xml");
        assert!(deserialized.display_name.is_none());
    }

    #[test]
    fn test_logo_collection_summary_response_serialization() {
        let summary = LogoCollectionSummaryResponse {
            id: uuid::Uuid::new_v4(),
            name: "Summary Collection".to_string(),
            description: std::option::Option::Some("Summary description".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            asset_count: 5,
            thumbnail_url: std::option::Option::Some("https://example.com/thumb.png".to_string()),
        };

        let serialized = serde_json::to_string(&summary).unwrap();
        let deserialized: LogoCollectionSummaryResponse = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.name, "Summary Collection");
        assert_eq!(deserialized.asset_count, 5);
        assert_eq!(deserialized.thumbnail_url.unwrap(), "https://example.com/thumb.png");
    }

    #[test]
    fn test_logo_collection_summary_without_thumbnail() {
        let summary = LogoCollectionSummaryResponse {
            id: uuid::Uuid::new_v4(),
            name: "No Thumbnail Collection".to_string(),
            description: std::option::Option::None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            asset_count: 0,
            thumbnail_url: std::option::Option::None,
        };

        let serialized = serde_json::to_string(&summary).unwrap();
        let deserialized: LogoCollectionSummaryResponse = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.asset_count, 0);
        assert!(deserialized.thumbnail_url.is_none());
        assert!(deserialized.description.is_none());
    }

    #[test]
    fn test_schema_round_trip_consistency() {
        // Test that all schemas can be serialized and deserialized consistently
        let create_request = CreateLogoCollectionRequest {
            name: "Round Trip Test".to_string(),
            description: std::option::Option::Some("Test description".to_string()),
            asset_ids: std::option::Option::Some(vec![uuid::Uuid::new_v4()]),
        };

        let serialized = serde_json::to_string(&create_request).unwrap();
        let deserialized: CreateLogoCollectionRequest = serde_json::from_str(&serialized).unwrap();
        let re_serialized = serde_json::to_string(&deserialized).unwrap();
        
        // The serialized forms should be equivalent
        let original_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        let round_trip_value: serde_json::Value = serde_json::from_str(&re_serialized).unwrap();
        assert_eq!(original_value, round_trip_value);
    }

    #[test]
    fn test_debug_implementations() {
        // Test that Debug implementations work properly
        let request = CreateLogoCollectionRequest {
            name: "Debug Test".to_string(),
            description: std::option::Option::Some("Debug description".to_string()),
            asset_ids: std::option::Option::None,
        };

        let debug_str = std::format!("{:?}", request);
        assert!(debug_str.contains("CreateLogoCollectionRequest"));
        assert!(debug_str.contains("Debug Test"));
    }
}
