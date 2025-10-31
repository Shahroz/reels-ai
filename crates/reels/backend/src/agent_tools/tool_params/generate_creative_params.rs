//! Defines parameters for the `generate_creative` agent tool.
//!
//! This structure encapsulates the request payload for generating a creative via an LLM,
//! containing IDs for all the necessary components like styles, assets, documents,
//! and creative formats.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct GenerateCreativeParams {
    pub name: std::string::String,
    pub collection_id: uuid::Uuid,
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub style_id: Option<uuid::Uuid>,
    pub asset_ids: Option<std::vec::Vec<uuid::Uuid>>,
    pub document_ids: Option<std::vec::Vec<uuid::Uuid>>,
    pub creative_format_ids: std::vec::Vec<uuid::Uuid>,
    pub bundle_ids: Option<std::vec::Vec<uuid::Uuid>>,
    #[schemars(skip)]
    pub organization_id: Option<uuid::Uuid>,
}
