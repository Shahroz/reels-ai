//! Defines parameters for creating a new user database collection.
//!
//! This structure encapsulates all necessary information for the
//! `create_user_db_collection` tool. It includes user identification,
//! the desired name for the new collection, an optional description,
//! and the initial JSON schema definition that will govern the structure
//! of items within this collection. Adheres to Rust coding guidelines.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct CreateUserDbCollectionParams {
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub initial_schema_definition: serde_json::Value,
}