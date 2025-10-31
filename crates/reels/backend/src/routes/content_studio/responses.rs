//! Response structures for content studio endpoints.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::documents::Document;

/// Response for document transformation and content generation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContentGenerationResponse {
    /// The generated or transformed document
    pub document: Document,
    
    /// ID of the source document (if this was a transformation)
    pub source_document_id: Option<Uuid>,
    
    /// The LLM model used for generation
    #[schema(example = "gpt_4o_mini")]
    pub model_used: String,
    
    /// Time taken for generation in milliseconds
    #[schema(example = 2500)]
    pub generation_time_ms: u64,
    
    /// Whether provenance tracking was established
    #[schema(example = true)]
    pub has_provenance: bool,
}

/// Node in a document lineage graph
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentLineageNode {
    /// Document ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub document_id: Uuid,
    
    /// Document title
    #[schema(example = "Original Research Document")]
    pub title: String,
    
    /// Document content preview (first 200 characters)
    #[schema(example = "This document contains important research findings about...")]
    pub content_preview: String,
    
    /// Creation timestamp
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Whether this is the root document
    pub is_root: bool,
}

/// Edge in a document lineage graph
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentLineageEdge {
    /// Source document ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub source_document_id: Uuid,
    
    /// Target document ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid")]
    pub target_document_id: Uuid,
    
    /// Type of relationship
    #[schema(example = "content_transformation")]
    pub relation_type: String,
    
    /// Additional parameters stored with the edge
    pub params: serde_json::Value,
    
    /// When the relationship was created
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Document lineage graph response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentLineageResponse {
    /// All nodes in the lineage graph
    pub nodes: Vec<DocumentLineageNode>,
    
    /// All edges in the lineage graph
    pub edges: Vec<DocumentLineageEdge>,
    
    /// The root document ID that was queried
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub root_document_id: Uuid,
    
    /// Total number of documents in the lineage
    #[schema(example = 5)]
    pub total_documents: u32,
    
    /// Maximum depth reached in the lineage
    #[schema(example = 3)]
    pub max_depth_reached: u32,
    
    /// Studio journey ID for this document lineage
    #[schema(example = "550e8400-e29b-41d4-a716-446655440002", format = "uuid")]
    pub journey_id: Option<Uuid>,
}

/// Available models response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AvailableModelsResponse {
    /// List of available LLM models
    #[schema(example = json!(["gpt_4o_mini", "gemini_25_flash", "claude_35_sonnet"]))]
    pub models: Vec<String>,
    
    /// Default model used if none specified
    #[schema(example = "gpt_4o_mini")]
    pub default_model: String,
}

/// Health check response for content generation service
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContentServiceHealthResponse {
    /// Whether the service is healthy
    pub healthy: bool,
    
    /// Number of available models
    #[schema(example = 3)]
    pub available_models_count: u32,
    
    /// Service configuration summary
    pub config_summary: String,
}

/// Response for listing template documents
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListTemplateDocumentsResponse {
    /// List of template documents
    pub templates: Vec<crate::db::documents::Document>,
    
    /// Total number of templates matching the query (for pagination)
    #[schema(example = 15)]
    pub total: i64,
    
    /// Number of templates returned in this response
    #[schema(example = 10)]
    pub count: i64,
    
    /// Offset used for pagination
    #[schema(example = 0)]
    pub offset: i64,
}
