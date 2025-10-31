//! Defines parameters for updating metadata of a user database collection.
//!
//! This structure is for the `update_user_db_collection` tool. It identifies
//! the user and the collection to be updated. It allows for optionally
//! changing the collection's name and/or its description. A double Option
//! for description allows distinguishing between not changing it, setting it
//! to a new string, or clearing it (setting to None).

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct UpdateUserDbCollectionParams {
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub collection_id_to_update: uuid::Uuid,
    pub new_name: Option<String>,
    pub new_description: Option<Option<String>>,
}