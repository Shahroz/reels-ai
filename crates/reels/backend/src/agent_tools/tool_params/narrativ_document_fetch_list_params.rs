//! Defines parameters for the Narrativ 'narrativ_document_fetch_list' tool.
//!
//! This struct holds parameters for fetching a list of documents, including pagination and sorting.
//! Adheres to Rust coding standards.

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, utoipa::ToSchema, Default)]
pub struct NarrativDocumentFetchListParams {
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,

    #[schema(example = "report")]
    pub search_pattern: std::option::Option<std::string::String>,

    #[schema(example = 10)]
    pub limit: std::option::Option<i64>,

    #[schema(example = 0)]
    pub offset: std::option::Option<i64>,

    #[schema(example = "created_at")]
    pub sort_by: std::option::Option<std::string::String>,

    #[schema(example = "desc")]
    pub sort_order: std::option::Option<std::string::String>,
}