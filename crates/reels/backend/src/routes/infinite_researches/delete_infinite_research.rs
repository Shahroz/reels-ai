//! Handler for deleting an infinite research task.

use crate::auth::tokens::Claims;
use crate::gcp::scheduler::SchedulerClient;
use crate::queries::infinite_research;
use crate::routes::error_response::ErrorResponse;
use actix_web::{delete, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/api/infinite-researches/{id}",
    tag = "Infinite Research",
    params(
        ("id" = Uuid, Path, description = "ID of the infinite research task to delete")
    ),
    responses(
        (status = 204, description = "Task deleted successfully"),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(("user_auth" = []))
)]
#[delete("/{id}")]
pub async fn delete_infinite_research(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let research_id = path.into_inner();
    let user_id = claims.user_id;

    // 1. Verify ownership and get scheduler job name
    let research = match infinite_research::get_infinite_research_by_id(pool.get_ref(), research_id, user_id).await {
        Ok(Some(r)) => r,
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Failed to find infinite research {research_id} for deletion: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to find task to delete.".into()});
        }
    };

    // 2. Delete the scheduler job from GCP
    if research.scheduler_job_name.is_some() {
        let scheduler_client = match SchedulerClient::new().await {
            Ok(client) => client,
            Err(e) => {
                log::error!("Failed to create SchedulerClient: {e}");
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to initialize scheduler.".into(),
                });
            }
        };

        let job_id = research.id.to_string();
        if let Err(e) = scheduler_client.delete_scheduler_job(&job_id).await {
            // Log the error but proceed with DB deletion. The job might already be gone.
            log::warn!("Failed to delete scheduler job '{job_id}': {e}. Proceeding with DB deletion.");
        }
    }

    // 3. Delete from the database
    match infinite_research::delete_infinite_research(pool.get_ref(), research_id, user_id).await {
        Ok(0) => HttpResponse::NotFound().finish(), // Should not happen due to check above
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::error!("Failed to delete infinite research {research_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to delete research task.".into(),
            })
        }
    }
}
