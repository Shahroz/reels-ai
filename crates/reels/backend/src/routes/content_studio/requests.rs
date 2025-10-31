//! Request structures for content studio endpoints.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Request to transform an existing document
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct TransformDocumentRequest {
    /// ID of the source document to transform
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub source_document_id: Uuid,
    
    /// Transformation prompt describing how to modify the content
    /// Optional when template_document_id is provided
    #[validate(length(min = 1, max = 5000, message = "Transformation prompt must be between 1 and 5000 characters"))]
    #[schema(example = "Rewrite this content in a more formal tone suitable for business communication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformation_prompt: Option<String>,
    
    /// ID of template document to use for transformation format
    /// Optional alternative to transformation_prompt
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_document_id: Option<Uuid>,
    
    /// Title for the generated document
    #[validate(length(min = 1, max = 255, message = "Title must be between 1 and 255 characters"))]
    #[schema(example = "Formal Version of Original Document")]
    pub target_title: String,
}

/// Request to generate new content from scratch
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct GenerateContentRequest {
    /// Content generation prompt
    #[validate(length(min = 1, max = 5000, message = "Prompt must be between 1 and 5000 characters"))]
    #[schema(example = "Write a comprehensive blog post about the benefits of artificial intelligence in healthcare")]
    pub prompt: String,
    
    /// Title for the generated document
    #[validate(length(min = 1, max = 255, message = "Title must be between 1 and 255 characters"))]
    #[schema(example = "AI in Healthcare: Transforming Patient Care")]
    pub title: String,
    
    /// Optional collection to associate the document with
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid")]
    pub collection_id: Option<Uuid>,
}

/// Request to get document lineage graph
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct DocumentLineageRequest {
    /// ID of the document to get lineage for
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub document_id: Uuid,
    
    /// Maximum depth of lineage to retrieve
    #[validate(range(min = 1, max = 10, message = "Depth must be between 1 and 10"))]
    #[schema(example = 5)]
    pub max_depth: Option<u32>,
}

/// Request parameters for listing template documents
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct ListTemplateDocumentsParams {
    /// Search query to filter templates by title or content
    #[schema(example = "invoice")]
    #[serde(default)]
    pub search: String,
    
    /// Maximum number of templates to return
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    #[schema(example = 20)]
    #[serde(default = "default_limit")]
    pub limit: i64,
    
    /// Number of templates to skip for pagination
    #[validate(range(min = 0, message = "Offset must be 0 or greater"))]
    #[schema(example = 0)]
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}
