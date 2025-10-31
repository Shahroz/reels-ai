// crates/narrativ/backend/src/routes/user_db_collections/items/query_user_db_collection_items_request.rs
// Purpose: Defines the request body for querying items in a user DB collection.

use serde::Deserialize;
use utoipa::ToSchema;

/// Request body for querying items in a user DB collection.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct QueryUserDbCollectionItemsRequest {
    /// The query string to filter items.
    /// Schema example: `title = "My Document" AND views > 100`
    #[schema(example = "title = \"My Document\" AND views > 100")]
    pub query: String,

    /// The page number for pagination.
    /// Defaults to 1.
    #[schema(example = 1, default = 1)]
    pub page: Option<i64>,

    /// The number of items per page for pagination.
    /// Defaults to 10.
    #[schema(example = 10, default = 10)]
    pub limit: Option<i64>,
}
