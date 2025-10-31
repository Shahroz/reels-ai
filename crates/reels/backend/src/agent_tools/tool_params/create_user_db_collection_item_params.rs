//! Defines parameters for creating a new item within a user database collection.
//!
//! This structure is used by the `create_user_db_collection_item` tool.
//! It specifies the user, the target collection's UUID, and the actual
//! item data as a JSON value. The item data must conform to the
//! schema defined for the target collection. Adheres to Rust standards.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct CreateUserDbCollectionItemParams {
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub collection_id_uuid: uuid::Uuid,
    pub item_data: serde_json::Value,
}