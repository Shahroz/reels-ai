//! Request structure for creating predefined collections.
//!
//! This module defines the request body structure for creating new predefined collections.
//! Adheres to 'one item per file' and FQN guidelines.

/// Request structure for creating a predefined collection.
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreatePredefinedCollectionRequest {
    #[schema(example = "Product Catalog")]
    pub name: String,
    #[schema(example = "A collection for managing product catalog data")]
    pub description: Option<String>,
    #[schema(example = r#"{"type": "object", "properties": {"name": {"type": "string"}, "price": {"type": "number"}}}"#)]
    pub schema_definition: serde_json::Value,
    #[schema(example = r#"{"type": "form", "fields": [{"name": "name", "type": "text"}, {"name": "price", "type": "number"}]}"#)]
    pub ui_component_definition: serde_json::Value,
} 