//! Handler for bulk deducting credit transactions via imageboard webhook API.
//!
//! POST /imageboard/webhook/{collection_id}/transactions
//! - Collection validation handled by ImageboardWebhookGuard middleware
//! - Gets transactions from request body
//! - Fetches collection to get organization_id
//! - Processes bulk credit deductions

use crate::schemas::imageboard_schemas::{
    BillingTransactionRecord, BulkDeductCreditsLiteResponse, WebhookErrorResponse,
};
use crate::services::imageboard_webhook_handler::handlers::credit_bulk_deduct_transaction_lite::handle_bulk_deduct_lite;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/imageboard/webhook/{collection_id}/transactions",
    tag = "Imageboard",
    params(
        ("collection_id" = uuid::Uuid, Path, description = "Collection ID (board_id) - organization_id is extracted from the collection")
    ),
    request_body = Vec<BillingTransactionRecord>,
    responses(
        (status = 200, description = "Bulk credit deduction processed", body = BulkDeductCreditsLiteResponse),
        (status = 400, description = "Invalid request", body = WebhookErrorResponse),
        (status = 404, description = "Organization or collection not found", body = WebhookErrorResponse),
        (status = 500, description = "Internal server error", body = WebhookErrorResponse)
    )
)]
#[actix_web::post("/{collection_id}/transactions")]
pub async fn bulk_deduct_credit_transaction(
    pool: web::Data<PgPool>,
    board_id: web::ReqData<Uuid>,
    payload: web::Json<Vec<BillingTransactionRecord>>,
) -> impl Responder {
    // Fetch collection to get organization_id
    let collection = match crate::queries::collections::get_collection_by_id::get_collection_by_id(
        pool.get_ref(),
        *board_id,
    )
    .await
    {
        Ok(Some(collection)) => collection,
        Ok(None) => {
            return HttpResponse::NotFound().json(WebhookErrorResponse {
                error: "Board not found".to_string(),
                code: "BOARD_NOT_FOUND".to_string(),
                details: None,
            });
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(WebhookErrorResponse {
                error: "Internal server error".to_string(),
                code: "INTERNAL_SERVER_ERROR".to_string(),
                details: Some("Failed to retrieve board".to_string()),
            });
        }
    };

    let organization_id = match collection.organization_id {
        Some(org_id) => org_id,
        None => {
            return HttpResponse::BadRequest().json(WebhookErrorResponse {
                error: "Board does not have an organization linked".to_string(),
                code: "BOARD_NO_ORGANIZATION".to_string(),
                details: Some(board_id.to_string()),
            });
        }
    };

    // Collection is already validated by ImageboardWebhookGuard middleware
    match handle_bulk_deduct_lite(pool.get_ref(), *board_id, organization_id, &payload)
        .await
    {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(err_resp) => HttpResponse::BadRequest().json(err_resp),
    }
}
