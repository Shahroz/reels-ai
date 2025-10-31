//! Defines parameters for the Narrativ 'narrativ_document_count' tool.
//!
//! This struct holds the user ID and an optional search pattern for counting documents.
//! Adheres to Rust coding standards: one item per file, FQNs.

#[derive(
    std::fmt::Debug,
    std::clone::Clone,
   std::default::Default,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    utoipa::ToSchema,
)]
pub struct NarrativDocumentCountParams {
    #[schema(example = "018f0f9b-5f51-7505-9023-43190b493574")]
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
    pub search_pattern: std::option::Option<std::string::String>,
}