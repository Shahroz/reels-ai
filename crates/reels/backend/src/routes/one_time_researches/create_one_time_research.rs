//! Handler for creating a new one-time research task.
//!
//! This endpoint accepts a prompt, creates a corresponding database record,
//! and queues a task in Google Cloud Tasks for asynchronous execution.

use crate::auth::task_token;
use crate::gcp::cloud_tasks::CloudTasksClient;
use crate::queries;
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse, Responder, HttpRequest};
use sqlx::PgPool;

use super::create_one_time_research_request::CreateOneTimeResearchRequest;

#[utoipa::path(
    post,
    path = "/api/one-time-researches",
    request_body = CreateOneTimeResearchRequest,
    params(
        ("x-organization-id" = Option<String>, Header, description = "Optional organization ID to deduct credits from organization instead of user")
    ),
    responses(
        (status = 201, description = "One-time research task created and queued", body = crate::db::one_time_research::OneTimeResearch),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("jwt_token" = [])
    ),
    tag = "One-Time Research"
)]
#[actix_web::post("")]
#[tracing::instrument(skip(pool, auth, payload, tasks_client))]
pub async fn create_one_time_research(
    pool: web::Data<PgPool>,
    auth: web::ReqData<crate::auth::tokens::Claims>,
    payload: web::Json<CreateOneTimeResearchRequest>,
    tasks_client: web::Data<CloudTasksClient>,
    req: HttpRequest,
) -> impl Responder {
    // 1. Create a database record for the research task.
    let research_task = match queries::one_time_research::create_one_time_research(
        pool.get_ref(),
        auth.user_id,
        &payload.prompt,
    )
    .await
    {
        Ok(task) => task,
       Err(e) => {
           log::error!("Failed to create one-time research in db: {e}");
           return HttpResponse::InternalServerError().json(ErrorResponse {
               error: "Failed to create research task.".to_string(),
           });
       }
   };

    // 2. Generate a short-lived JWT for the task worker to authenticate.
    let task_jwt = match task_token::generate_task_jwt(research_task.id) {
        Ok(token) => token,
        Err(e) => {
            log::error!(
                "Failed to generate task JWT for research_id {}: {}",
                research_task.id,
               e
           );
           return HttpResponse::InternalServerError().json(ErrorResponse {
               error: "Failed to create task token.".to_string(),
           });
       }
   };

    // 3. Enqueue the task in Google Cloud Tasks.
    let task_id_str = research_task.id.to_string();
    let relative_uri = format!("/api/internal/run-one-time-research/{task_id_str}");

    // Build extra headers if x-organization-id header is provided
    let mut extra_headers = std::collections::HashMap::new();
    if let Some(org_id) = req
        .headers()
        .get("x-organization-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
    {
        extra_headers.insert("x-organization-id".to_string(), org_id);
    }
    let extra_headers_option = if extra_headers.is_empty() {
        None
    } else {
        Some(&extra_headers)
    };

    let cloud_task = match tasks_client
        .create_http_task_with_jwt(
            &task_id_str,
            &relative_uri,
            bytes::Bytes::new(), // Body is empty, auth is in header
            &task_jwt,
            extra_headers_option,
        )
        .await
    {
        Ok(task) => task,
        Err(e) => {
            log::error!(
                "Failed to create cloud task for research_id {}: {}",
                research_task.id,
               e
           );
           return HttpResponse::InternalServerError().json(ErrorResponse {
               error: "Failed to queue research task.".to_string(),
           });
       }
   };

    // 4. Update the DB record with the cloud task name and "queued" status.
   let updated_task = match queries::one_time_research::update_one_time_research_status(
       pool.get_ref(),
       research_task.id,
       "queued",
       Some(&cloud_task.name),
   )
   .await
   {
        Ok(task) => task,
        Err(e) => {
            log::error!(
                "Failed to update status for research_id {}: {}",
                research_task.id,
               e
           );
           // The task is queued but our DB state is inconsistent. This is an internal error.
           return HttpResponse::InternalServerError().json(ErrorResponse {
               error: "Failed to update research task status after queuing.".to_string(),
           });
       }
   };

    HttpResponse::Created().json(updated_task)
}
