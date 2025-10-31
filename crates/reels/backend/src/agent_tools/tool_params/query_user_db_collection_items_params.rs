//! Defines parameters for querying items within a user database collection using a query string.
//!
//! This structure is for the `query_user_db_collection_items` tool. It allows
//! for complex queries against items in a specified collection, identified by
//! user and collection ID. Supports pagination (page, limit) for query results.
//! The `query_string` allows for flexible, potentially rich querying syntax.
//! Compliant with Rust coding guidelines.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct QueryUserDbCollectionItemsParams {
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub collection_id: uuid::Uuid,
    pub query_string: String,
    pub page: i64,
    pub limit: i64,
}