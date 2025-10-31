//! Handler for updating an existing infinite research task.

use crate::auth::{scheduler_token, tokens::Claims};
use crate::gcp::scheduler::SchedulerClient;
use crate::queries::infinite_research::{self, InfiniteResearch};
use crate::routes::error_response::ErrorResponse;
use crate::routes::infinite_researches::update_infinite_research_request::UpdateInfiniteResearchRequest;
use actix_web::{put, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    put,
    path = "/api/infinite-researches/{id}",
    tag = "Infinite Research",
    params(
        ("id" = Uuid, Path, description = "ID of the infinite research task to update")
    ),
    request_body = UpdateInfiniteResearchRequest,
    responses(
        (status = 200, description = "Infinite research task updated", body = InfiniteResearch),
        (status = 400, description = "Validation Error", body = ErrorResponse),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(("user_auth" = []))
)]
#[put("/{id}")]
pub async fn update_infinite_research(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateInfiniteResearchRequest>,
) -> impl Responder {
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation failed: {e}"),
        });
    }

    let research_id = path.into_inner();
    let user_id = claims.user_id;

    // Ensure the task exists and the user owns it
    let existing_research = match infinite_research::get_infinite_research_by_id(pool.get_ref(), research_id, user_id).await {
        Ok(Some(research)) => research,
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to verify task ownership".into()}),
    };

    let job_id = existing_research.id.to_string();
    let scheduler_job_name = existing_research.scheduler_job_name.clone().unwrap_or_else(|| {
        // This case should be rare, but we can construct the name if missing.
        let project_id = std::env::var("GCP_PROJECT_ID").unwrap_or_default();
        let location = std::env::var("GCP_LOCATION").unwrap_or_default();
        format!("projects/{project_id}/locations/{location}/jobs/{job_id}")
    });
    
    // Update scheduler job on GCP
    let scheduler_client = match SchedulerClient::new().await {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to create SchedulerClient: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to initialize scheduler.".into(),
            });
        }
    };

    let jwt_token = scheduler_token::generate_scheduler_jwt(user_id, research_id);
    let description = format!("Infinite Research task: {}", req.name);
    let time_zone = "UTC";

    // TODO: Add logic to pause/resume job if is_enabled changes, instead of just updating.
    // The current update will effectively re-enable it.
    if let Err(e) = scheduler_client
        .update_scheduler_job(&job_id, &description, &req.cron_schedule, time_zone, &jwt_token)
        .await
    {
        log::error!("Failed to update scheduler job: {e}");
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to update scheduled job.".into(),
        });
    }

    // Update DB record
    match infinite_research::update_infinite_research(
        pool.get_ref(),
        research_id,
        user_id,
        &req.name,
        &req.prompt,
        &req.cron_schedule,
        req.is_enabled,
        Some(&scheduler_job_name),
    ).await {
        Ok(updated_research) => HttpResponse::Ok().json(updated_research),
        Err(e) => {
            log::error!("Failed to update infinite research record: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to update research task in database.".into(),
            })
        }
    }
}
