//! Handler for listing the execution history of an infinite research task.

use crate::auth::tokens::Claims;
use crate::db::infinite_research_execution::{self, InfiniteResearchExecution};
use crate::routes::error_response::ErrorResponse;
use crate::queries::infinite_research;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/infinite-researches/{id}/executions",
    tag = "Infinite Research",
    params(
        ("id" = Uuid, Path, description = "ID of the infinite research task")
    ),
    responses(
        (status = 200, description = "List of task executions", body = Vec<InfiniteResearchExecution>),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(("user_auth" = []))
)]
#[get("/{id}/executions")]
pub async fn list_infinite_research_executions(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let research_id = path.into_inner();
    let user_id = claims.user_id;

    // First, verify the user has access to the parent research task.
    if let Err(e) = infinite_research::get_infinite_research_by_id(pool.get_ref(), research_id, user_id).await {
        return match e {
            sqlx::Error::RowNotFound => HttpResponse::NotFound().finish(),
            _ => {
                 log::error!("Failed to verify ownership for research {research_id}: {e}");
                HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to verify task ownership".into()})
            }
        };
    }

    match infinite_research_execution::list_executions_by_research_id(pool.get_ref(), research_id).await {
        Ok(executions) => HttpResponse::Ok().json(executions),
        Err(e) => {
            log::error!("Failed to list executions for research {research_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve task executions.".into(),
            })
        }
    }
}
