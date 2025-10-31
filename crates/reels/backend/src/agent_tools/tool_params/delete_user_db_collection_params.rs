//! Defines parameters for deleting an existing user database collection.
//!
//! This structure is used by the `delete_user_db_collection` tool.
//! It requires the ID of the user performing the action and the unique
//! identifier of the database collection that is targeted for deletion.
//! Ensures adherence to Rust coding standards for modularity.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct DeleteUserDbCollectionParams {
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub collection_id_to_delete: uuid::Uuid,
}