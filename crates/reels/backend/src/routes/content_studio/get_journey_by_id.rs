//! Get studio journey by journey ID endpoint.

use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;
use crate::queries::documents::lineage::get_or_create_document_node::get_document_nodes_for_journey;
use crate::routes::content_studio::create_document_journey::DocumentJourneyResponse;
use crate::routes::content_studio::get_document_journey::JourneyNodeResponse;

/// Extended journey response with nodes - by journey ID
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct DocumentJourneyByIdResponse {
    pub journey: DocumentJourneyResponse,
    pub nodes: Vec<JourneyNodeResponse>,
}

#[utoipa::path(
    get,
    path = "/api/content-studio/journey-by-id/{journey_id}",
    tag = "Content Studio",
    params(
        ("journey_id" = Uuid, Path, description = "Journey ID to get")
    ),
    responses(
        (status = 200, description = "Journey retrieved successfully", body = DocumentJourneyByIdResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Journey not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[get("/journey-by-id/{journey_id}")]
#[instrument(skip(pool, claims))]
pub async fn get_journey_by_id(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let journey_id = path.into_inner();
    let user_id = claims.user_id;
    
    tracing::info!(
        "🔍 GET_JOURNEY_BY_ID START: journey_id={}, user_id={}",
        journey_id,
        user_id
    );

    // First, verify this journey belongs to the user
    let journey_record = match sqlx::query!(
        r#"SELECT id, user_id, root_asset_id as root_document_id, name 
           FROM studio_journeys 
           WHERE id = $1 AND user_id = $2"#,
        journey_id,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(record)) => {
            tracing::info!(
                "✅ JOURNEY FOUND: journey_id={}, user_id={}, root_document_id={:?}, name={:?}",
                record.id, record.user_id, record.root_document_id, record.name
            );
            record
        },
        Ok(None) => {
            tracing::warn!(
                "❌ JOURNEY NOT FOUND: journey_id={}, user_id={}",
                journey_id,
                user_id
            );
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Journey not found".to_string(),
            });
        }
        Err(e) => {
            tracing::error!(
                "💥 JOURNEY QUERY ERROR: journey_id={}, user_id={}, error={}",
                journey_id,
                user_id,
                e
            );
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch journey".to_string(),
            });
        }
    };

    // Get all nodes for this journey
    tracing::info!("🔍 FETCHING NODES for journey_id={}", journey_id);
    let nodes = match get_document_nodes_for_journey(pool.get_ref(), journey_id).await {
        Ok(nodes) => {
            tracing::info!(
                "✅ NODES FETCHED: journey_id={}, node_count={}, nodes={:?}",
                journey_id, nodes.len(), nodes
            );
            nodes
        },
        Err(e) => {
            tracing::error!(
                "💥 NODES QUERY ERROR: journey_id={}, user_id={}, error={}",
                journey_id,
                user_id,
                e
            );
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to get journey nodes".to_string(),
            });
        }
    };

    let journey_response = DocumentJourneyResponse {
        journey_id: journey_record.id,
        user_id: journey_record.user_id,
        root_document_id: journey_record.root_document_id,
        name: journey_record.name,
    };
    
    let node_responses: Vec<JourneyNodeResponse> = nodes.into_iter().map(|node| {
        JourneyNodeResponse {
            node_id: node.id,
            document_id: node.document_id,
            parent_node_id: node.parent_node_id,
            custom_prompt: node.custom_prompt,
        }
    }).collect();
    
    let response = DocumentJourneyByIdResponse {
        journey: journey_response,
        nodes: node_responses,
    };
    
    tracing::info!(
        "🎉 RESPONSE READY: journey_id={}, node_count={}, response={:?}",
        journey_id,
        response.nodes.len(),
        response
    );
    
    HttpResponse::Ok().json(response)
}
