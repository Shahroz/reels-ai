//! Get document studio journey endpoint.

use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;
use crate::queries::documents::lineage::get_or_create_document_journey::get_or_create_document_journey;
use crate::queries::documents::lineage::get_or_create_document_node::{get_document_nodes_for_journey, get_or_create_document_node};
use crate::routes::content_studio::create_document_journey::DocumentJourneyResponse;

/// Node information in journey response
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct JourneyNodeResponse {
    pub node_id: Uuid,
    pub document_id: Uuid,
    pub parent_node_id: Option<Uuid>,
    pub custom_prompt: Option<String>,
}

/// Extended journey response with nodes
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct DocumentJourneyWithNodesResponse {
    pub journey: DocumentJourneyResponse,
    pub nodes: Vec<JourneyNodeResponse>,
}

#[utoipa::path(
    get,
    path = "/api/content-studio/journey/{document_id}",
    tag = "Content Studio",
    params(
        ("document_id" = Uuid, Path, description = "Document ID to get journey for")
    ),
    responses(
        (status = 200, description = "Journey retrieved successfully", body = DocumentJourneyWithNodesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Journey not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[get("/journey/{document_id}")]
#[instrument(skip(pool, claims))]
pub async fn get_document_journey_by_document_id(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    document_id: web::Path<Uuid>,
) -> impl Responder {
    let user_id = claims.user_id;
    let document_id = document_id.into_inner();
    
    tracing::info!(
        "Getting journey for document {} for user {}",
        document_id,
        user_id
    );

    // Try to get existing journey, or create one if it doesn't exist
    match get_or_create_document_journey(
        pool.get_ref(),
        user_id,
        document_id,
        None, // No custom name when getting existing journey
    ).await {
        Ok(journey) => {
            // Get all nodes for this journey
            match get_document_nodes_for_journey(
                pool.get_ref(),
                journey.id,
            ).await {
                Ok(mut nodes) => {
                    // If the journey has no nodes, add the initial document as a node
                    if nodes.is_empty() {
                        tracing::info!("Journey has no nodes, adding initial document as node");
                        match get_or_create_document_node(
                            pool.get_ref(),
                            journey.id,
                            document_id,
                            None, // No parent for initial document
                            None, // No custom prompt
                        ).await {
                            Ok(initial_node) => {
                                tracing::info!("Successfully added initial document node: {}", initial_node.id);
                                nodes.push(initial_node);
                            }
                            Err(e) => {
                                tracing::error!("Failed to add initial document node: {}", e);
                                // Continue anyway, will return empty journey
                            }
                        }
                    }
                    let journey_response = DocumentJourneyResponse {
                        journey_id: journey.id,
                        user_id: journey.user_id,
                        root_document_id: journey.root_document_id,
                        name: journey.name,
                    };
                    
                    let node_responses: Vec<JourneyNodeResponse> = nodes.into_iter().map(|node| {
                        JourneyNodeResponse {
                            node_id: node.id,
                            document_id: node.document_id,
                            parent_node_id: node.parent_node_id,
                            custom_prompt: node.custom_prompt,
                        }
                    }).collect();
                    
                    let response = DocumentJourneyWithNodesResponse {
                        journey: journey_response,
                        nodes: node_responses,
                    };
                    
                    tracing::info!(
                        "Successfully retrieved journey {} with {} nodes for document {}",
                        journey.id,
                        response.nodes.len(),
                        document_id
                    );
                    
                    HttpResponse::Ok().json(response)
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to get nodes for journey {} for document {}: {}",
                        journey.id,
                        document_id,
                        e
                    );
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to retrieve journey nodes".to_string(),
                    })
                }
            }
        }
        Err(e) => {
            tracing::error!(
                "Failed to get/create journey for document {} for user {}: {}",
                document_id,
                user_id,
                e
            );
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve or create journey".to_string(),
            })
        }
    }
}
