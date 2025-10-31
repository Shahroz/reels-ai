//! Defines parameters for the `generate_style_from_url` agent tool.
//!
//! This structure captures the necessary inputs for generating a new style
//! from a given URL, including a name for the new style and the source URL
//! to be processed.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct GenerateStyleFromUrlParams {
   pub name: String,
   pub source_url: String,
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    /// Optional organization ID to deduct credits from organization instead of user
    #[schemars(skip)]
    pub organization_id: Option<uuid::Uuid>,
}