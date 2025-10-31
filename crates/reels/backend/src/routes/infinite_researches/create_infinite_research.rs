//! Handler for creating a new infinite research task.

use crate::auth::{scheduler_token, tokens::Claims};
use crate::gcp::scheduler::SchedulerClient;
use crate::queries::infinite_research::{self, InfiniteResearch};
use crate::routes::error_response::ErrorResponse;
use crate::routes::infinite_researches::create_infinite_research_request::CreateInfiniteResearchRequest;
use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/infinite-researches",
    tag = "Infinite Research",
    request_body = CreateInfiniteResearchRequest,
    responses(
        (status = 201, description = "Infinite research task created", body = InfiniteResearch),
        (status = 400, description = "Validation Error", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(("user_auth" = []))
)]
#[post("")]
pub async fn create_infinite_research(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    req: web::Json<CreateInfiniteResearchRequest>,
) -> impl Responder {
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation failed: {e}"),
        });
    }

    let user_id = claims.user_id;

    // 1. Create the initial DB record
    let initial_research = match infinite_research::create_infinite_research(
        pool.get_ref(),
        user_id,
        &req.name,
        &req.prompt,
        &req.cron_schedule,
        None, // No scheduler job name yet
    )
    .await
    {
        Ok(research) => research,
        Err(e) => {
            log::error!("Failed to create initial infinite research record: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create research task.".into(),
            });
        }
    };

    // 2. Create the scheduler job on GCP
    let scheduler_client = match SchedulerClient::new().await {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to create SchedulerClient: {e}");
            // Attempt to clean up the created DB record
            let _ = infinite_research::delete_infinite_research(pool.get_ref(), initial_research.id, user_id).await;
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to initialize scheduler.".into(),
            });
        }
    };

    let jwt_token = scheduler_token::generate_scheduler_jwt(user_id, initial_research.id);
    let job_id = initial_research.id.to_string();
    let description = format!("Infinite Research task: {}", req.name);
    // TODO: Make timezone configurable per user/org
    let time_zone = "UTC";

    let scheduler_job_name = match scheduler_client
        .create_scheduler_job(&job_id, &description, &req.cron_schedule, time_zone, &jwt_token)
        .await
    {
        Ok(name) => name,
        Err(e) => {
            log::error!("Failed to create scheduler job: {e}");
            // Attempt to clean up the created DB record
            let _ = infinite_research::delete_infinite_research(pool.get_ref(), initial_research.id, user_id).await;
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create scheduled job.".into(),
            });
        }
    };

    // 3. Update the DB record with the scheduler job name
    match infinite_research::update_infinite_research(
        pool.get_ref(),
        initial_research.id,
        user_id,
        &initial_research.name,
        &initial_research.prompt,
        &initial_research.cron_schedule,
        initial_research.is_enabled,
        Some(&scheduler_job_name),
    ).await {
        Ok(final_research) => HttpResponse::Created().json(final_research),
        Err(e) => {
            log::error!("Failed to update research with scheduler job name: {e}");
            // Attempt to clean up both DB and scheduler job
             let _ = infinite_research::delete_infinite_research(pool.get_ref(), initial_research.id, user_id).await;
             let _ = scheduler_client.delete_scheduler_job(&job_id).await;
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to finalize research task creation.".into(),
            })
        }
    }
}
