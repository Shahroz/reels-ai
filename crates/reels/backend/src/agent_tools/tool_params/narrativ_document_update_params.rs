//! Defines parameters for the Narrativ 'narrativ_document_update' tool.
//!
//! This struct holds the document ID, user ID, title, content, and visibility options for updating an existing document.
//! Enhanced to support public documents and research inclusion settings for feature parity with HTTP routes.
//! Adheres to Rust coding standards with fully qualified paths.

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, utoipa::ToSchema, std::default::Default)]
pub struct NarrativDocumentUpdateParams {
    #[schema(example = "018f0f9a-7e6a-7c6c-92f8-093a0f9a7e6a")]
    pub document_id: uuid::Uuid,
    #[schema(example = "018f0f9a-7e6b-7c6d-92f9-093a0f9a7e6b")]
    #[schemars(skip)]
    pub user_id: std::option::Option<uuid::Uuid>,
    #[schema(example = "Updated Document Title")]
    pub title: std::string::String,
    #[schema(example = "This is the new, updated content of the document.")]
    pub content: std::string::String,
    #[schema(example = "true")]
    #[serde(default)]
    pub is_public: std::option::Option<bool>,
    #[schema(example = "true")]
    #[serde(default)]
    pub is_task: std::option::Option<bool>,
    #[serde(default)]
    pub include_research: std::option::Option<crate::db::document_research_usage::DocumentResearchUsage>,
}