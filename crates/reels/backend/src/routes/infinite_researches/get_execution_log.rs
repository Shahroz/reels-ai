//! Handler for retrieving the log file for an infinite research execution.

use crate::services::gcs::gcs_operations::GCSOperations;
use serde_json::json;
use crate::auth::tokens::Claims;
use crate::db::infinite_research_execution;
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/infinite-researches/executions/{id}/log",
    tag = "Infinite Research",
    params(
        ("id" = Uuid, Path, description = "ID of the infinite research execution")
    ),
    responses(
        (status = 200, description = "Execution log content", content_type = "application/json", body = String),
        (status = 404, description = "Execution or log not found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(("user_auth" = []))
)]
#[get("/executions/{id}/log")]
#[tracing::instrument(skip(pool, gcs_client, claims))]
pub async fn get_execution_log(
    pool: web::Data<PgPool>,
    gcs_client: web::Data<Arc<dyn GCSOperations>>,
    claims: web::ReqData<Claims>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let execution_id = path.into_inner();
    let user_id = claims.user_id;

    let execution = match infinite_research_execution::get_execution_by_id_and_user_id(
        pool.get_ref(),
        execution_id,
        user_id,
    )
    .await
    {
        Ok(Some(exec)) => exec,
        Ok(None) => return HttpResponse::NotFound().json(ErrorResponse {
            error: "Execution not found or you don't have access.".into(),
        }),
        Err(e) => {
            log::error!("Failed to fetch execution {execution_id}: {e}");
            return HttpResponse::InternalServerError().json(json!{{"error": e.to_string()}});
        }
    };

    let log_url: String = if let Some(url) = execution.output_log {
        url
    } else {
        return HttpResponse::NotFound().json(ErrorResponse {
            error: "Execution log not found for this execution.".into(),
        });
    };

    let (bucket, object) = match crate::services::gcs::parse_gcs_url::parse_gcs_url(&log_url) {
        Ok(res) => res,
        Err(e) => {
            log::error!("Failed to parse GCS URL '{log_url}': {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to process log location.".into(),
            });
        }
    };

    match gcs_client
        .download_object_as_string(&bucket, &object)
        .await
    {
        Ok(log_content) => HttpResponse::Ok()
            .content_type("application/json")
            .body(log_content),
        Err(e) => {
            log::error!("Failed to download log from GCS '{log_url}': {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve log file.".into(),
            })
        }
    }
}