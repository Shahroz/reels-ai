//! Request structure for updating predefined collections.
//!
//! This module defines the request body structure for updating existing predefined collections.
//! Adheres to 'one item per file' and FQN guidelines.

/// Request structure for updating a predefined collection.
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdatePredefinedCollectionRequest {
    #[schema(example = "Updated Product Catalog")]
    pub name: Option<String>,
    #[schema(example = "An updated collection for managing product catalog data")]
    pub description: Option<String>,
    #[schema(example = r#"{"type": "object", "properties": {"name": {"type": "string"}, "price": {"type": "number"}, "category": {"type": "string"}}}"#)]
    pub schema_definition: Option<serde_json::Value>,
    #[schema(example = r#"{"type": "form", "fields": [{"name": "name", "type": "text"}, {"name": "price", "type": "number"}, {"name": "category", "type": "select"}]}"#)]
    pub ui_component_definition: Option<serde_json::Value>,
} 