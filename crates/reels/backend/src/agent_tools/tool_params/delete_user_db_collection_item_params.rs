//! Defines parameters for deleting an item from a user database collection.
//!
//! For use with the `delete_user_db_collection_item` tool, this structure
//! identifies the user, the collection containing the item, and the unique
//! UUID of the item to be deleted. This ensures precise removal of
//! specific data entries. Complies with Rust coding guidelines.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct DeleteUserDbCollectionItemParams {
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub collection_id_uuid: uuid::Uuid,
    pub item_id_uuid: uuid::Uuid,
}