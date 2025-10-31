//! Defines parameters for retrieving a specific item from a user database collection.
//!
//! This structure supports the `get_user_db_collection_item` tool. It requires
//! the user's ID, the UUID of the collection, and the UUID of the specific
//! item to be fetched. This allows for direct access to individual
//! data records within a collection. Adheres to Rust best practices.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct GetUserDbCollectionItemParams {
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub collection_id_uuid: uuid::Uuid,
    pub item_id_uuid: uuid::Uuid,
}