//! Defines parameters for the Narrativ 'narrativ_document_delete' tool.
//!
//! This struct holds the document ID and user ID for deleting a document.
//! Adheres to Rust coding standards.

// Private helper function for `schemars::JsonSchema` example.
// While `rust_guidelines` prefer one primary item per file, this function
// is a small, ancillary helper directly supporting the schema definition
// of `NarrativDocumentDeleteParams` and is not a standalone logical item.

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, utoipa::ToSchema, Default)]
pub struct NarrativDocumentDeleteParams {
    #[schema(example = "018f0f9b-5f51-7505-9023-43190b493574")]
    pub document_id: uuid::Uuid,
    #[schema(example = "018f0f9b-5f51-7505-9023-43190b493574")]
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
}