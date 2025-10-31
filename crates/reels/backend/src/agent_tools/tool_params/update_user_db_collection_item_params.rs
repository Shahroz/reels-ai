
//! Defines parameters for merging data into an existing item in a user database collection.
//!
//! This structure supports a merge/patch operation via the `update_user_db_collection_item` tool.
//! It identifies an item by its UUID and provides a JSON patch. Fields from the patch
//! are added to or overwrite fields in the item, preserving any not present in the patch.
//! The final merged item must conform to the collection's schema.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct UpdateUserDbCollectionItemParams {
    pub user_id: Option<uuid::Uuid>,
    pub collection_id_uuid: uuid::Uuid,
    pub item_id_uuid: uuid::Uuid,
    /// A JSON object with fields to merge into the existing item.
    /// This performs a patch-like operation, not a full replacement.
    pub item_data_patch: serde_json::Value,
}
