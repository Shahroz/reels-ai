//! Defines parameters for the `list_collections` agent tool.
//!
//! This structure encapsulates the query parameters for listing collections,
//! including pagination, sorting, and search functionality. It is used
//! for strong typing in the tool's implementation and for generating
//! a JSON schema for the agent's tool definition.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct ListCollectionsParams {
   pub page: Option<i64>,
   pub limit: Option<i64>,
   pub sort_by: Option<String>,
   pub sort_order: Option<String>,
    pub search: Option<String>,
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
}