//! Defines the response body for querying items in a user DB collection.
//!
//! This struct encapsulates the paginated list of items retrieved
//! based on the user's query, along with the total count of matching items.
//! Adheres to 'one item per file' and FQN guidelines.

/// Response body for querying items in a user DB collection.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct QueryUserDbCollectionItemsResponse {
    /// The list of items matching the query for the current page.
    pub items: std::vec::Vec<crate::db::user_db_collection_item::UserDbCollectionItem>,
    /// The total number of items matching the query across all pages.
    #[schema(example = 100)]
    pub total_count: i64,
}
