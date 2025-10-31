//! Defines parameters for listing user database collections with pagination and sorting.
//!
//! This structure supports the `list_user_db_collections` tool, allowing
//! for controlled fetching of multiple collections belonging to a user.
//! It includes parameters for pagination (limit, offset), sorting criteria
//! (column name, order), and a search pattern for filtering results based
//! on collection metadata. Follows Rust coding conventions.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct ListUserDbCollectionsParams {
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub limit: i64,
    pub offset: i64,
    pub sort_by_db_col_name: String,
    pub sort_order_db: String,
    pub search_pattern_db: Option<String>,
}