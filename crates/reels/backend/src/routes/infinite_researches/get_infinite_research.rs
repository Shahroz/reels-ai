//! Handler for retrieving a single infinite research task by its ID.

use crate::auth::tokens::Claims;
use crate::queries::infinite_research::{self, InfiniteResearch};
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/infinite-researches/{id}",
    tag = "Infinite Research",
    params(
        ("id" = Uuid, Path, description = "ID of the infinite research task")
    ),
    responses(
        (status = 200, description = "Infinite research task details", body = InfiniteResearch),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(("user_auth" = []))
)]
#[get("/{id}")]
pub async fn get_infinite_research(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let research_id = path.into_inner();
    let user_id = claims.user_id;

    match infinite_research::get_infinite_research_by_id(pool.get_ref(), research_id, user_id).await {
        Ok(Some(research)) => HttpResponse::Ok().json(research),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Failed to get infinite research {research_id} for user {user_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve research task.".into(),
            })
        }
    }
}
