//! Handler for executing an infinite research task via a scheduler call.
//!
//! This endpoint is protected by a custom JWT, not the standard user session JWT.

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use agentloop::types::research_request::ResearchRequest;

/// Defines the structure of the custom JWT claims for infinite research jobs.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct InfiniteResearchClaims {
    pub user_id: uuid::Uuid,
    pub infinite_research_id: uuid::Uuid,
}

#[utoipa::path(
    post,
    path = "/api/internal/run-infinite-research/{id}",
    params(
        ("id" = Uuid, Path, description = "The ID of the infinite research task to run.")
    ),
    responses(
        (status = 200, description = "Research task executed successfully."),
        (status = 400, description = "Bad request, e.g., missing or invalid token."),
        (status = 401, description = "Unauthorized, e.g., token verification failed or claims mismatch."),
        (status = 404, description = "Infinite research task not found."),
        (status = 500, description = "Internal server error.")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Internal",
)]
#[actix_web::post("run-infinite-research/{id}")]
#[instrument(skip_all, fields(infinite_research_id = %id))]
pub async fn run_infinite_research(
    pool: web::Data<sqlx::PgPool>,
    gcs_client: web::Data<crate::services::gcs::gcs_client::GCSClient>,
    // app_state is needed for full agent execution, keeping it for future use.
    // _app_state: web::Data<agentloop::state::app_state::AppState>,
    req: HttpRequest,
    id: web::Path<uuid::Uuid>,
) -> impl Responder {
    let infinite_research_id = id.into_inner();

    // 1. Extract and verify the custom JWT from the Authorization header.
    let token = match req.headers().get("Authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")) {
        Some(token) => token,
        None => {
            tracing::warn!("Missing or malformed Authorization header");
            return HttpResponse::BadRequest().body("Missing or malformed Authorization header.");
        }
    };

    let jwt_secret = match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            tracing::error!("JWT_SECRET environment variable not set");
            return HttpResponse::InternalServerError().body("Server configuration error.");
        }
    };

    let claims = match crate::utils::jwt::verify_jwt_token::<InfiniteResearchClaims>(token, &jwt_secret) {
        Ok(claims) => claims,
        Err(e) => {
            tracing::warn!("JWT verification failed: {}", e);
            return HttpResponse::Unauthorized().body("Invalid or expired token.");
        }
    };

    // 2. Validate claims against the request.
    if claims.infinite_research_id != infinite_research_id {
        tracing::warn!(
            "Token claim mismatch: URL ID {} does not match token ID {}",
            infinite_research_id,
            claims.infinite_research_id
        );
        return HttpResponse::Unauthorized().body("Token-ID mismatch.");
    }

    // 3. Fetch the research task and verify ownership.
    let research_task = match crate::queries::infinite_research::get_infinite_research_by_id::get_infinite_research_by_id(&pool, infinite_research_id, claims.user_id).await {
        Ok(Some(task)) => task,
        Ok(None) => {
            tracing::warn!("Infinite research task not found or user mismatch for ID: {}", infinite_research_id);
            return HttpResponse::NotFound().finish();
        }
        Err(e) => {
            tracing::error!("Failed to fetch infinite research task: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Ensure the task is enabled
    if !research_task.is_enabled {
        tracing::info!("Skipping disabled infinite research task: {}", research_task.id);
        // Return 200 OK because the job itself is valid, just not active.
        return HttpResponse::Ok().body("Task is disabled, execution skipped.");
    }

    // 4. Create an execution record.
    let execution_record = match crate::db::infinite_research_execution::create_execution(&pool, research_task.id, "running").await {
        Ok(record) => record,
        Err(e) => {
            tracing::error!("Failed to create execution record: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // 5. Extract organization_id from header if present
    let organization_id = req.headers()
        .get("x-organization-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());

    // 6. Execute the research agent logic.
    tracing::info!("Executing synchronous agent task for prompt: '{}'", research_task.prompt);

    let research_request = ResearchRequest {
        user_id: claims.user_id,
        instruction: research_task.prompt.clone(),
        attachments: None,
        organization_id,
    };

    let research_result = crate::services::agent_service::run_and_log_research(
        research_request,
        &gcs_client,
        pool.get_ref(),
        execution_record.id,
    )
    .await;

   let (final_status, output_log, error_message) = match research_result {
       Ok(log_url) => (
           "completed",
           Some(log_url),
           None,
       ),
       Err(e) => ("failed", None, Some(e)),
    };

   // 7. Update the execution record with the final status and output.
   if let Err(e) = crate::db::infinite_research_execution::update_execution_on_finish(
       &pool,
       execution_record.id,
       final_status,
       output_log,
       error_message,
   ).await {
       tracing::error!("Failed to update final execution status: {}", e);
       // The job ran, but we couldn't log the result. Still return 500.
        return HttpResponse::InternalServerError().body("Failed to finalize execution record.");
    }

    tracing::info!("Successfully completed infinite research task execution for ID: {}", research_task.id);
    HttpResponse::Ok().body("Execution completed.")
}
