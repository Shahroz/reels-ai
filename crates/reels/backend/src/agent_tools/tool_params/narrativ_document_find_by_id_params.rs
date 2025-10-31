//! Defines parameters for the Narrativ 'narrativ_document_find_by_id' tool.
//!
//! This struct holds the document ID and user ID for finding a specific document.
//! Adheres to Rust coding standards.

#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    utoipa::ToSchema,
   Default,
)]
pub struct NarrativDocumentFindByIdParams {
    #[schema(example = "018f0f9b-5f51-7505-9023-43190b493574")]
    pub document_id: uuid::Uuid,
    #[schema(example = "018f0f9b-6f51-7505-9023-43190b493574")]
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
}