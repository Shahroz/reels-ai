//! Defines parameters for the `list_styles` agent tool.
//!
//! This structure encapsulates the query parameters for listing styles,
//! including pagination, sorting, search, and filtering by favorite status.
//! It is used for strong typing in the tool's implementation and for
//! generating a JSON schema for the agent's tool definition.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct ListStylesParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub search: Option<String>,
    pub is_favorite: Option<bool>,
    pub user_id: Option<uuid::Uuid>
}