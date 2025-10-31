//! Defines parameters for updating the schema of a user database collection.
//!
//! This file contains the `UpdateUserDbCollectionSchemaParams` struct for identifying
//! the user and collection, and specifying the schema update payload.
//! The `UpdateUserDbCollectionSchemaPayload` enum allows the schema to be
//! updated either directly with a JSON value or via an instruction string
//! for schema generation. Compliant with Rust coding guidelines.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct UpdateUserDbCollectionSchemaParams {
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub collection_id: uuid::Uuid,
    pub payload: UpdateUserDbCollectionSchemaPayload,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum UpdateUserDbCollectionSchemaPayload {
    Direct(serde_json::Value),
    Instruction(String),
}

impl Default for UpdateUserDbCollectionSchemaPayload {
    fn default() -> Self {
        UpdateUserDbCollectionSchemaPayload::Direct(serde_json::Value::Null)
    }
}