//! Defines the request payload for creating or updating a custom creative format.
//!
//! This structure captures the necessary fields from the client.

use crate::db::creative_type::CreativeType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateCustomFormatRequest {
    #[schema(example = "Custom Banner 100x250")]
    pub name: String,
    #[schema(example = "Optional description for the custom format")]
    pub description: Option<String>,
    #[schema(example = 100)]
    pub width: Option<i32>, // Make optional
    #[schema(example = 250)]
    pub height: Option<i32>, // Make optional
    #[schema(value_type = String, example = "image")]
    pub creative_type: CreativeType, // Add type
    #[schema(value_type = Object, nullable = true, example = json!({"schema": "details"}))]
    pub json_schema: Option<serde_json::Value>, // Add schema
    #[schema(example = false)]
    pub is_public: Option<bool>, // Add optional isPublic field
    pub metadata: Option<serde_json::Value>,
}