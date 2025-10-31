//! Defines the payload for creating a new document task.
//!
//! This struct represents the JSON body expected by the API when
//! submitting a new document request. It contains fields for title,
//! content, and optional sources.
//! Documents the schema example for OpenAPI generation.
use crate::db::document_research_usage::DocumentResearchUsage;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)] // Add Validate to derive
pub struct CreateDocumentRequest {
    #[schema(example = "Competitor Analysis: Acme Corp")]
    #[validate(length(min = 1, message = "Title cannot be empty"))] // Add validation rule
    pub title: String,

    #[schema(example = "A deep dive into Acme Corp's recent product launches.")]
    #[validate(length(min = 1, message = "Content cannot be empty"))] // Add validation rule
    pub content: String,

    #[schema(example = json!(["https://acme.com/news", "https://techreport.io/acme-analysis"]), nullable = true)]
   pub sources: Option<Vec<String>>, // Allow optional sources on creation

    #[schema(example = false, default = false, nullable = true)]
    pub is_task: Option<bool>,

    #[schema(example = "TaskDependent", nullable = true)] // Relies on DocumentResearchUsage::ToSchema for enum type definition
    pub include_research: Option<DocumentResearchUsage>,

    #[schema(example = false, default = false, nullable = true)]
    pub is_public: Option<bool>,
}
