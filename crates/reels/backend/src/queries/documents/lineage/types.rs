use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentTransformationParams {
    /// Prompt used for document transformation
    pub transformation_prompt: String,
    /// Original document title for context
    pub source_document_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum DocumentDerivationParams {
    ContentTransformation(DocumentTransformationParams),
    ContentGeneration,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentGraphNode {
    pub document_id: sqlx::types::Uuid,
    pub title: String,
    pub content_preview: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>, 
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentGraphEdge {
    pub source_document_id: sqlx::types::Uuid,
    pub derived_document_id: sqlx::types::Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentGraph {
    pub nodes: Vec<DocumentGraphNode>,
    pub edges: Vec<DocumentGraphEdge>,
    pub root_document_id: sqlx::types::Uuid,
    /// Journey ID in database (if journey has been created)
    pub journey_id: Option<sqlx::types::Uuid>,
}
