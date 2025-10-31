//! Handler for manually triggering an infinite research task.

use crate::gcp::scheduler::SchedulerClient;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;
use crate::routes::error_response::ErrorResponse;
use crate::auth::tokens::Claims;

#[utoipa::path(
    post,
    path = "/api/infinite-researches/{id}/run",
    params(
        ("id" = Uuid, Path, description = "ID of the infinite research task to run")
    ),
    responses(
        (status = 202, description = "Accepted - The job run has been triggered."),
        (status = 404, description = "Task not found"),
        (status = 400, description = "Bad Request - Task has no scheduler job associated with it"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Infinite Research"
)]
#[actix_web::post("/{id}/run")]
#[instrument(skip(pool, claims))]
pub async fn run_infinite_research_manually(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let research_id = id.into_inner();
    let user_id = claims.user_id;

    // 2. Create the scheduler job on GCP
    let scheduler_client = match SchedulerClient::new().await {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to create SchedulerClient: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create SchedulerClient".into(),
            })
        }
    };

    let research_task = match crate::queries::infinite_research::get_infinite_research_by_id::get_infinite_research_by_id(&pool, research_id, user_id).await {
        Ok(Some(task)) => task,
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(e) => {
            tracing::error!("Failed to fetch infinite research task: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if let Some(job_name) = research_task.scheduler_job_name {
        // The job name from GCP is the full path, e.g. projects/your-project/locations/your-loc/jobs/job-id
        // Our client expects just the job-id.
        if let Some(job_id) = job_name.split('/').next_back() {
            match scheduler_client.run_scheduler_job(job_id).await {
                Ok(_) => HttpResponse::Accepted().body("Job run triggered."),
                Err(e) => {
                    tracing::error!("Failed to trigger scheduler job '{}': {}", job_id, e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        } else {
            tracing::error!("Could not parse job_id from scheduler_job_name: {}", job_name);
            HttpResponse::InternalServerError().body("Could not parse job ID from scheduler job name.")
        }
    } else {
        HttpResponse::BadRequest().body("This research task does not have a scheduler job associated with it.")
    }
}
