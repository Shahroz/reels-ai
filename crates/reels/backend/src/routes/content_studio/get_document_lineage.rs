//! Get document lineage route handler.
//!
//! This handler retrieves the lineage graph for a document, showing its derivation history
//! and transformation relationships.

use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::tokens::Claims;
use crate::queries::documents::lineage::get_graph_for_document::get_graph_for_document;
use crate::queries::documents::lineage::get_or_create_document_journey::get_or_create_document_journey;
use crate::routes::content_studio::responses::{DocumentLineageResponse, DocumentLineageNode, DocumentLineageEdge};
use crate::routes::error_response::ErrorResponse;

#[derive(Deserialize, ToSchema)]
pub struct DocumentLineageParams {
    /// ID of the document to get lineage for
    pub document_id: String,
}

/// Get document lineage graph
#[utoipa::path(
    get,
    path = "/api/content-studio/document-lineage/{document_id}",
    params(
        ("document_id" = String, Path, description = "Document ID to get lineage for")
    ),
    responses(
        (status = 200, description = "Document lineage retrieved successfully", body = DocumentLineageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Document not found"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Content Studio",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_document_lineage(
    path: web::Path<DocumentLineageParams>,
    pool: web::Data<sqlx::PgPool>,
    claims: Claims,
) -> Result<HttpResponse> {
    let document_id = match Uuid::parse_str(&path.document_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid document ID format: Document ID must be a valid UUID".to_string(),
            }));
        }
    };

    tracing::info!("Getting lineage for document {} by user {}", document_id, claims.user_id);
    
    match get_graph_for_document(&**pool, claims.user_id, document_id).await {
        Ok(document_graph) => {
            // Ensure studio journey exists in database (like Image Studio does)
            // Use the requested document_id, not root_document_id, like Image Studio uses original.id
            match get_or_create_document_journey(&**pool, claims.user_id, document_id, None).await {
                Ok(journey) => {
                    // Convert DocumentGraph to API response format
                    let nodes: Vec<DocumentLineageNode> = document_graph.nodes.into_iter().map(|node| {
                        DocumentLineageNode {
                            document_id: node.document_id,
                            title: node.title,
                            content_preview: node.content_preview,
                            created_at: node.created_at.unwrap_or_else(chrono::Utc::now),
                            is_root: node.document_id == document_graph.root_document_id,
                        }
                    }).collect();

                    let edges: Vec<DocumentLineageEdge> = document_graph.edges.into_iter().map(|edge| {
                        DocumentLineageEdge {
                            source_document_id: edge.source_document_id,
                            target_document_id: edge.derived_document_id,
                            relation_type: "content_transformation".to_string(),
                            params: serde_json::Value::Null,
                            created_at: chrono::Utc::now(), // We don't have this in the graph
                        }
                    }).collect();

                    let total_documents = nodes.len() as u32;
                    let response = DocumentLineageResponse {
                        nodes,
                        edges,
                        root_document_id: document_graph.root_document_id,
                        total_documents,
                        max_depth_reached: 1, // We don't calculate depth in the current implementation
                        journey_id: Some(journey.id), // Include journey_id like Image Studio does
                    };

                    Ok(HttpResponse::Ok().json(response))
                }
                Err(e) => {
                    tracing::error!("get_or_create_document_journey error: {}", e);
                    Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to create or retrieve document journey".to_string(),
                    }))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to get document lineage: {}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to retrieve document lineage: {}", e),
            }))
        }
    }
}
