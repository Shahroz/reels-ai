//! Defines the payload for updating document content.
//!
//! This struct represents the JSON body expected by the API when
//! updating the content of an existing document entry.

use crate::db::document_research_usage::DocumentResearchUsage;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct UpdateDocumentRequest {
    #[schema(example = "Updated Document Title", nullable = true)]
    #[validate(length(min = 1, message = "Title cannot be empty if provided"))]
    pub title: Option<String>,
    #[schema(example = "Updated document content here.", nullable = true)]
    #[validate(length(min = 1, message = "Content cannot be empty if provided"))]
    pub content: Option<String>,
    #[schema(example = true, nullable = true)]
    pub is_task: Option<bool>,
    #[schema(example = "Always", nullable = true)] // Relies on DocumentResearchUsage::ToSchema for enum type definition
    pub include_research: Option<DocumentResearchUsage>,
    #[schema(example = true, default = false, nullable = true)]
    pub is_public: Option<bool>,
}
