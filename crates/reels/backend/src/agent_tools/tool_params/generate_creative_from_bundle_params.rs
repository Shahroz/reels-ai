//! Defines parameters for the `generate_creative_from_bundle` agent tool.
//!
//! This structure captures the request payload for generating a creative
//! from a specified bundle, including the creative's name, the collection it
//! belongs to, the bundle to use, and an optional list of specific documents.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct GenerateCreativeFromBundleParams {
   pub name: std::string::String,
   #[schemars(skip)]
   pub user_id: Option<uuid::Uuid>,
   pub collection_id: uuid::Uuid,
   pub bundle_id: uuid::Uuid,
   pub document_ids: Option<std::vec::Vec<uuid::Uuid>>,
   #[schemars(skip)]
   pub organization_id: Option<uuid::Uuid>,
}