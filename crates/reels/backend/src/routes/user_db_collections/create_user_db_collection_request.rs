//! Defines the request body for creating a new user DB collection.
//!
//! This struct contains the necessary information to define a new collection,
//! including its name, an optional description, and the JSON schema for its items.
//! Adheres to 'one item per file' and FQN guidelines.

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateUserDbCollectionRequest {
    #[schema(example = "My Photo Album")]
    pub name: String,
    #[schema(example = "A collection of holiday photos.")]
    pub description: Option<String>,
    #[schema(value_type = Object, example = json!({"type": "object", "properties": {"url": {"type": "string"}, "caption": {"type": "string"}}}))]
    pub schema_definition: serde_json::Value,
}
