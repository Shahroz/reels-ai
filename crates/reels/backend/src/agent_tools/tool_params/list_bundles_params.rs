//! Defines the parameters for the `list_bundles` agent tool.
//!
//! This structure holds the arguments for listing all expanded creative bundles
//! for the current user. The `user_id` is injected by the dispatcher and not
//! expected from the agent.
//! Adheres to the project's Rust coding standards.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct ListBundlesParams {
    /// The user ID, injected by the application. Not expected from the agent.
    pub user_id: Option<sqlx::types::Uuid>,
    /// How many bundles to return. Defaults to 20.
    pub limit: Option<i64>,
    /// The page number of results to return. Defaults to 0.
    pub page: Option<i64>,
    /// The field to sort by. Defaults to "created_at".
    pub sort_by: Option<String>,
    /// The sort order. Defaults to "desc".
    pub sort_order: Option<String>,
    /// A search string to filter bundles by name.
    pub search: Option<String>,
}
