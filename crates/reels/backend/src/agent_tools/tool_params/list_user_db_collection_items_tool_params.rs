//! Defines parameters for listing items within a user database collection.
//!
//! Used by the `list_user_db_collection_items` tool, this structure
//! facilitates paginated, sorted, and filtered retrieval of items.
//! It requires user and collection identification, pagination controls (page, limit),
//! sorting details (column name, order), and a search pattern for text-based
//! filtering of item content. Follows Rust coding conventions.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct ListUserDbCollectionItemsToolParams {
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub collection_id_uuid: uuid::Uuid,
    pub page: i64,
    pub limit: i64,
    pub sort_by_column_name: String,
    pub sort_order: String,
    pub search_pattern: Option<String>,
}
