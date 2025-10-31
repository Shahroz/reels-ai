//! Create or get document studio journey endpoint.

use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;
use crate::queries::documents::lineage::get_or_create_document_journey::get_or_create_document_journey;

/// Request to create or get a document journey
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct CreateDocumentJourneyRequest {
    /// ID of the root document for this journey
    pub root_document_id: Uuid,
    /// Optional name for the journey
    pub name: Option<String>,
}

/// Response containing journey information
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct DocumentJourneyResponse {
    pub journey_id: Uuid,
    pub user_id: Uuid,
    pub root_document_id: Option<Uuid>,
    pub name: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/content-studio/journey",
    tag = "Content Studio",
    request_body = CreateDocumentJourneyRequest,
    responses(
        (status = 200, description = "Journey created or retrieved successfully", body = DocumentJourneyResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[post("/journey")]
#[instrument(skip(pool, claims))]
pub async fn create_document_journey(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    req: web::Json<CreateDocumentJourneyRequest>,
) -> impl Responder {
    let user_id = claims.user_id;
    
    tracing::info!(
        "Creating/getting document journey for user {} with root document {}",
        user_id,
        req.root_document_id
    );

    match get_or_create_document_journey(
        pool.get_ref(),
        user_id,
        req.root_document_id,
        req.name.clone(),
    ).await {
        Ok(journey) => {
            tracing::info!(
                "Successfully created/retrieved journey {} for user {}",
                journey.id,
                user_id
            );
            
            let response = DocumentJourneyResponse {
                journey_id: journey.id,
                user_id: journey.user_id,
                root_document_id: journey.root_document_id,
                name: journey.name,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!(
                "Failed to create/get document journey for user {} with root document {}: {}",
                user_id,
                req.root_document_id,
                e
            );
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create or retrieve journey".to_string(),
            })
        }
    }
}
