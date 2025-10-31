//! Handler for the internal endpoint to run a one-time research task.
//!
//! This endpoint is called by Google Cloud Tasks. It is authenticated via a
//! short-lived, task-specific JWT.

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::task_token;
use crate::queries::one_time_research::{
    get_one_time_research_by_id_internal, update_one_time_research_on_finish,
    update_one_time_research_on_start,
};
use crate::services::agent_service;
use crate::services::gcs::gcs_client::GCSClient;

/// Extracts the Bearer token from the Authorization header.
fn get_token_from_request(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

#[actix_web::post("/run-one-time-research/{id}")]
#[tracing::instrument(skip(req, pool, gcs_client))]
pub async fn run_one_time_research(
    req: HttpRequest,
    id: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    gcs_client: web::Data<GCSClient>,
) -> impl Responder {
    let task_id = id.into_inner();

    // 1. Authenticate the request using the task JWT
    let token = match get_token_from_request(&req) {
        Some(token) => token,
        None => {
            log::warn!("Missing Authorization header for task {task_id}");
            return HttpResponse::Unauthorized().body("Missing Authorization header");
        }
    };

    let claims = match task_token::verify_task_jwt(&token) {
        Ok(claims) => claims,
        Err(e) => {
            log::warn!("Invalid JWT for task {task_id}: {e}");
            return HttpResponse::Unauthorized().body("Invalid JWT");
        }
    };

    // 2. Authorize: Ensure the JWT is for this specific task
    if claims.one_time_research_id != task_id {
        log::error!(
            "JWT-ID mismatch. Path ID: {}, Token ID: {}",
            task_id,
            claims.one_time_research_id
        );
        return HttpResponse::Forbidden().body("JWT does not match task ID");
    }

    // 3. Fetch the task details from the database
    let research_task = match get_one_time_research_by_id_internal(pool.get_ref(), task_id).await {
        Ok(task) => task,
        Err(e) => {
            log::error!("Failed to fetch research task {task_id}: {e}");
            // Return 500 to signal Cloud Tasks to retry
            return HttpResponse::InternalServerError().body("Failed to fetch task");
        }
    };

    // Prevent re-running a completed or failed task
    if research_task.status == "completed" || research_task.status == "failed" {
        log::info!("Task {} already in terminal state '{}'. Acknowledging to prevent retry loop.", task_id, research_task.status);
        return HttpResponse::Ok().body("Task already completed");
    }

    // 4. Update task status to 'running'
    if let Err(e) = update_one_time_research_on_start(pool.get_ref(), task_id).await {
        log::error!("Failed to update task {task_id} to running: {e}");
        return HttpResponse::InternalServerError().body("Failed to update task status");
    }

   // 5. Extract organization_id from header if present
   let organization_id = req.headers()
       .get("x-organization-id")
       .and_then(|v| v.to_str().ok())
       .and_then(|s| uuid::Uuid::parse_str(s).ok());

   // 6. Execute the research task
   log::info!("Starting research for task {}", task_id);
   let research_request = agentloop::types::research_request::ResearchRequest {
       user_id: research_task.user_id,
       instruction: research_task.prompt,
       attachments: Some(vec![]),
       organization_id,
   };

    let result = agent_service::run_and_log_research(
        research_request,
        gcs_client.get_ref(),
        pool.get_ref(),
        task_id,
    )
    .await;

    // 6. Update task status on finish
    let final_status_result = match result {
        Ok(output_log_url) => {
            log::info!("Task {task_id} completed successfully. Log: {output_log_url}");
            update_one_time_research_on_finish(
                pool.get_ref(),
                task_id,
                Some(&output_log_url),
                None,
            )
            .await
        }
        Err(e) => {
            log::error!("Task {task_id} failed: {e}");
            update_one_time_research_on_finish(
                pool.get_ref(),
                task_id,
                None,
                Some(&e.to_string()),
            )
            .await
        }
    };

    match final_status_result {
        Ok(_) => {
            log::info!("Final status for task {task_id} updated successfully.");
            // 200 OK signals to Cloud Tasks that the task is complete and should not be retried.
            HttpResponse::Ok().body("Task finished")
        }
        Err(e) => {
            log::error!(
                "CRITICAL: Failed to update final status for task {task_id}: {e}. This may cause a retry loop."
            );
            // Return 500 to signal a problem and potentially trigger a retry.
            HttpResponse::InternalServerError().body("Failed to update final task status")
        }
    }
}
