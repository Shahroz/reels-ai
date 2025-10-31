//! Defines parameters for the `list_formats` agent tool.
//!
//! This structure encapsulates the query parameters for listing custom creative formats,
//! including pagination, sorting, search, and filtering by public status.
//! It is used for strong typing in the tool's implementation and for generating
//! a JSON schema for the agent's tool definition.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct ListFormatsParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort_by: Option<String>,
   pub sort_order: Option<String>,
   pub search: Option<String>,
   pub is_public: Option<bool>,
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
}