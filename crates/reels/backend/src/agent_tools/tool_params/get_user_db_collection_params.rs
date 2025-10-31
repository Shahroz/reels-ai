//! Defines parameters for retrieving an existing user database collection.
//!
//! Used by the `get_user_db_collection` tool, this structure specifies
//! the user context and the unique identifier of the collection to be
//! fetched. This allows for targeted retrieval of a single collection's
//! metadata and schema. Complies with established Rust guidelines.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct GetUserDbCollectionParams {
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub collection_id_to_fetch: uuid::Uuid,
}