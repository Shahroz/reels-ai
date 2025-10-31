//! Defines parameters for the Narrativ 'narrativ_document_insert' tool.
//!
//! This structure is used to specify the data required when an agent needs to insert a new document
//! into the Narrativ document store. It includes fields for user identification, document metadata (title),
//! the main content of the document, and an optional list of URIs pointing to the sources of the information.

use crate::db::document_research_usage::DocumentResearchUsage;

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, utoipa::ToSchema, std::default::Default)]
pub struct NarrativDocumentInsertParams {
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    #[schema(example = "Title example")]
    pub title: std::string::String,
    #[schema(example = "Content example")]
    pub content: std::string::String,
    pub sources: std::option::Option<std::vec::Vec<std::string::String>>,
    #[serde(default)]
    pub is_public: std::option::Option<bool>,
    #[serde(default)]
    pub is_task: std::option::Option<bool>,
    #[serde(default)]
    pub include_research: std::option::Option<DocumentResearchUsage>,
}
