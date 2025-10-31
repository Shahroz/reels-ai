//! Handler for applying multiple watermarks to an asset in a single operation.
//!
//! POST /api/watermark/apply-batch

use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::instrument;

use crate::schemas::watermark_schemas::{ApplyBatchWatermarkRequest, WatermarkResponse};
use crate::services::watermarking::{apply_batch_watermark_sync_photon, watermark_error::WatermarkError};
use crate::services::gcs::gcs_operations::GCSOperations;
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    post,
    path = "/api/watermark/apply-batch",
    tag = "Watermarking",
    request_body = ApplyBatchWatermarkRequest,
    responses(
        (status = 200, description = "Success", body = WatermarkResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Asset not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[post("/apply-batch")]
#[instrument(skip(pool, gcs_client, payload, claims))]
pub async fn apply_batch_watermark(
    pool: web::Data<PgPool>,
    gcs_client: web::Data<Arc<dyn GCSOperations>>,
    payload: web::Json<ApplyBatchWatermarkRequest>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
) -> impl Responder {
    let user_id = claims.user_id;
    let request = payload.into_inner();

    tracing::info!(
        "Processing batch watermark request for user: {}, source: {}, watermarks: {}",
        user_id,
        request.source_asset_id,
        request.watermarks.len()
    );

    let result = apply_batch_watermark_sync_photon(
        pool.get_ref(),
        gcs_client.get_ref(),
        user_id,
        request.source_asset_id,
        request.watermarks,
    )
    .await;

    match result {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(WatermarkError::AssetNotFound(asset_id)) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: format!("Asset not found: {}", asset_id),
            })
        }
        Err(WatermarkError::InvalidConfig(msg)) => {
            HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid watermark configuration: {}", msg),
            })
        }
        Err(err) => {
            log::error!("Batch watermarking error: {:?}", err);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to apply batch watermark".to_string(),
            })
        }
    }
}
