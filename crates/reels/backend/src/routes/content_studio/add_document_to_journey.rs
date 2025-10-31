//! Add document to studio journey endpoint.

use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;
use crate::queries::documents::lineage::get_or_create_document_journey::get_document_journey;
use crate::queries::documents::lineage::get_or_create_document_node::get_or_create_document_node;
use crate::queries::documents::lineage::insert_document_provenance_edge::insert_document_provenance_edge;
use crate::queries::documents::lineage::types::{DocumentDerivationParams, DocumentTransformationParams};

/// Request to add a document to a journey
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct AddDocumentToJourneyRequest {
    /// Journey ID to add the document to
    pub journey_id: Uuid,
    /// Document ID to add
    pub document_id: Uuid,
    /// Parent document ID if this is a derived document
    pub parent_document_id: Option<Uuid>,
    /// Transformation prompt if this is a transformation
    pub transformation_prompt: Option<String>,
}

/// Response containing node information
#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct DocumentNodeResponse {
    pub node_id: Uuid,
    pub journey_id: Uuid,
    pub document_id: Uuid,
    pub parent_node_id: Option<Uuid>,
    pub has_provenance: bool,
}

#[utoipa::path(
    post,
    path = "/api/content-studio/journey/add-document",
    tag = "Content Studio",
    request_body = AddDocumentToJourneyRequest,
    responses(
        (status = 200, description = "Document added to journey successfully", body = DocumentNodeResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Journey access denied"),
        (status = 404, description = "Journey not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[post("/journey/add-document")]
#[instrument(skip(pool, claims))]
pub async fn add_document_to_journey(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    req: web::Json<AddDocumentToJourneyRequest>,
) -> impl Responder {
    let user_id = claims.user_id;
    
    tracing::info!(
        "Adding document {} to journey {} for user {}",
        req.document_id,
        req.journey_id,
        user_id
    );

    // Verify the user owns this journey
    match get_document_journey(
        pool.get_ref(),
        req.journey_id,
        user_id,
    ).await {
        Ok(Some(_)) => {
            // Journey exists and user has access
        }
        Ok(None) => {
            tracing::warn!(
                "Journey {} not found or access denied for user {}",
                req.journey_id,
                user_id
            );
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Journey not found or access denied".to_string(),
            });
        }
        Err(e) => {
            tracing::error!(
                "Database error when checking journey {} for user {}: {}",
                req.journey_id,
                user_id,
                e
            );
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Database error while accessing journey".to_string(),
            });
        }
    }

    // Add detailed logging to trace the parent_node_id issue
    tracing::info!(
        "🔍 ADD_DOCUMENT_TO_JOURNEY: Adding document {} to journey {} with parent_document_id: {:?}",
        req.document_id,
        req.journey_id,
        req.parent_document_id
    );

    // Create the node with proper parent relationship (like Image Studio)
    let parent_node_id = if let Some(parent_doc_id) = req.parent_document_id {
        tracing::info!(
            "🔍 PARENT_DOC_ID provided: {} - ensuring parent node exists in journey {}",
            parent_doc_id,
            req.journey_id
        );
        
        // Ensure parent document has a node in this journey first (like Image Studio does)
        match get_or_create_document_node(
            pool.get_ref(),
            req.journey_id,
            parent_doc_id,
            None, // Parent document has no parent in this journey
            None, // No transformation prompt for parent
        ).await {
            Ok(parent_node) => {
                tracing::info!(
                    "✅ PARENT_NODE created/found: node_id={}, document_id={}, journey_id={}",
                    parent_node.id,
                    parent_doc_id,
                    req.journey_id
                );
                Some(parent_node.id)
            }
            Err(e) => {
                tracing::error!(
                    "❌ FAILED to create/get parent node for document {} in journey {}: {}",
                    parent_doc_id,
                    req.journey_id,
                    e
                );
                None
            }
        }
    } else {
        tracing::info!(
            "🔍 NO parent_document_id provided - creating root node for document {}",
            req.document_id
        );
        None
    };

    tracing::info!(
        "🔍 CREATING_CHILD_NODE: document_id={}, journey_id={}, parent_node_id={:?}",
        req.document_id,
        req.journey_id,
        parent_node_id
    );

    match get_or_create_document_node(
        pool.get_ref(),
        req.journey_id,
        req.document_id,
        parent_node_id,
        req.transformation_prompt.clone(),
    ).await {
        Ok(node) => {
            tracing::info!(
                "✅ CHILD_NODE created: node_id={}, document_id={}, parent_node_id={:?}, journey_id={}",
                node.id,
                req.document_id,
                node.parent_node_id,
                req.journey_id
            );
            
            let mut has_provenance = false;
            
            // Create provenance edge if this is a transformation
            if let (Some(parent_doc_id), Some(prompt)) = (&req.parent_document_id, &req.transformation_prompt) {
                let derivation_params = DocumentDerivationParams::ContentTransformation(
                    DocumentTransformationParams {
                        transformation_prompt: prompt.clone(),
                        source_document_title: None, // TODO: Get source document title if needed
                    }
                );
                
                match insert_document_provenance_edge(
                    pool.get_ref(),
                    *parent_doc_id,
                    req.document_id,
                    derivation_params,
                ).await {
                    Ok(_) => {
                        has_provenance = true;
                        tracing::info!(
                            "Created provenance edge: {} -> {}",
                            parent_doc_id,
                            req.document_id
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to create provenance edge: {} -> {}: {}",
                            parent_doc_id,
                            req.document_id,
                            e
                        );
                        // Continue anyway, as the node was created successfully
                    }
                }
            }
            
            tracing::info!(
                "Successfully added document {} to journey {} as node {}",
                req.document_id,
                req.journey_id,
                node.id
            );
            
            let response = DocumentNodeResponse {
                node_id: node.id,
                journey_id: node.journey_id,
                document_id: node.document_id,
                parent_node_id: node.parent_node_id,
                has_provenance,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!(
                "Failed to add document {} to journey {}: {}",
                req.document_id,
                req.journey_id,
                e
            );
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to add document to journey".to_string(),
            })
        }
    }
}
