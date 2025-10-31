//! Handler for getting organization balance via imageboard webhook API.
//!
//! GET /imageboard/webhook/{collection_id}/balance
//! - Collection validation handled by ImageboardWebhookGuard middleware
//! - Fetches collection to get organization_id
//! - Returns organization credit balance

use sqlx::PgPool;
use actix_web::{web, HttpResponse, Responder};
use crate::schemas::imageboard_schemas::{OrganizationCreditsAllocationLiteResponse, WebhookErrorResponse};
use crate::services::imageboard_webhook_handler::handlers::organization_balance_fetch::handle_org_balance_fetch;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/imageboard/webhook/{collection_id}/balance",
    tag = "Imageboard",
    params(
        ("collection_id" = uuid::Uuid, Path, description = "Collection ID (board_id) - organization_id is extracted from the collection")
    ),
    responses(
        (status = 200, description = "Balance fetched successfully", body = OrganizationCreditsAllocationLiteResponse),
        (status = 400, description = "Invalid request", body = WebhookErrorResponse),
        (status = 404, description = "Organization not found", body = WebhookErrorResponse),
        (status = 500, description = "Internal server error", body = WebhookErrorResponse)
    )
)]
#[actix_web::get("/{collection_id}/balance")]
pub async fn get_organization_balance(
    pool: web::Data<PgPool>,
    board_id: web::ReqData<Uuid>,
) -> impl Responder {
    // Fetch collection to get organization_id
    let collection = match crate::queries::collections::get_collection_by_id::get_collection_by_id(
        pool.get_ref(),
        *board_id,
    ).await {
        Ok(Some(collection)) => collection,
        Ok(None) => {
            return HttpResponse::NotFound().json(crate::schemas::imageboard_schemas::WebhookErrorResponse {
                error: "Board not found".to_string(),
                code: "BOARD_NOT_FOUND".to_string(),
                details: None,
            });
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(crate::schemas::imageboard_schemas::WebhookErrorResponse {
                error: "Internal server error".to_string(),
                code: "INTERNAL_SERVER_ERROR".to_string(),
                details: Some("Failed to retrieve board".to_string()),
            });
        }
    };

    let organization_id = match collection.organization_id {
        Some(org_id) => org_id,
        None => {
            return HttpResponse::BadRequest().json(crate::schemas::imageboard_schemas::WebhookErrorResponse {
                error: "Board does not have an organization linked".to_string(),
                code: "BOARD_NO_ORGANIZATION".to_string(),
                details: Some(board_id.to_string()),
            });
        }
    };

    // Collection is already validated by ImageboardWebhookGuard middleware
    match handle_org_balance_fetch(pool.get_ref(), organization_id).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(err_resp) => {
            if err_resp.code == "ORGANIZATION_NOT_FOUND" || err_resp.code == "ORG_CREDIT_ALLOCATION_NOT_FOUND" {
                HttpResponse::NotFound().json(err_resp)
            } else {
                HttpResponse::BadRequest().json(err_resp)
            }
        }
    }
}

