//! Handler for updating the enabled status of an infinite research task.

use crate::auth::tokens::Claims;
use crate::gcp::scheduler::SchedulerClient;
use crate::queries::infinite_research::{self, InfiniteResearch};
use crate::routes::error_response::ErrorResponse;
use crate::routes::infinite_researches::update_infinite_research_status_request::UpdateInfiniteResearchStatusRequest;
use actix_web::{patch, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    patch,
    path = "/api/infinite-researches/{id}/status",
    tag = "Infinite Research",
    params(
        ("id" = Uuid, Path, description = "ID of the infinite research task to update")
    ),
    request_body = UpdateInfiniteResearchStatusRequest,
    responses(
        (status = 200, description = "Infinite research task status updated", body = InfiniteResearch),
        (status = 400, description = "Validation Error", body = ErrorResponse),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(("user_auth" = []))
)]
#[patch("/{id}/status")]
pub async fn update_infinite_research_status(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateInfiniteResearchStatusRequest>,
) -> impl Responder {
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation failed: {e}"),
        });
    }

    let research_id = path.into_inner();
    let user_id = claims.user_id;

    let existing_research = match infinite_research::get_infinite_research_by_id(pool.get_ref(), research_id, user_id).await {
        Ok(Some(research)) => research,
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to verify task ownership".into()}),
    };

    let job_id = existing_research.id.to_string();
    
    // Pause or resume scheduler job on GCP
    let scheduler_client = match SchedulerClient::new().await {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to create SchedulerClient: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to initialize scheduler.".into(),
            });
        }
    };

    let scheduler_result = if req.is_enabled {
        scheduler_client.resume_scheduler_job(&job_id).await
    } else {
        scheduler_client.pause_scheduler_job(&job_id).await
    };

    if let Err(e) = scheduler_result {
        log::error!("Failed to update scheduler job status: {e}");
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to update scheduled job status.".into(),
        });
    }

    // Update DB record
    match infinite_research::update_infinite_research_status(
        pool.get_ref(),
        research_id,
        user_id,
        req.is_enabled,
    ).await {
        Ok(updated_research) => HttpResponse::Ok().json(updated_research),
        Err(e) => {
            log::error!("Failed to update infinite research status in DB: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to update research task status in database.".into(),
            })
        }
    }
}
