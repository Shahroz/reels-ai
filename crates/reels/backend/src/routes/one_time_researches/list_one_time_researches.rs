//! Handler for listing all one-time research tasks for a user.
//!
//! This endpoint provides a complete history of a user's one-time research tasks,
//! effectively serving as a list of their executions. Each item includes the
//! current status and other relevant details.

use crate::auth::tokens::Claims;
use crate::db::one_time_research::OneTimeResearch;
use crate::queries;
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;

#[utoipa::path(
    get,
    path = "/api/one-time-researches",
    tag = "One-Time Research",
    responses(
        (status = 200, description = "A list of one-time research tasks.", body = Vec<OneTimeResearch>),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(("jwt_token" = [])),
)]
#[get("")]
#[tracing::instrument(skip(pool, claims))]
pub async fn list_one_time_researches(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let user_id = claims.user_id;

    match queries::one_time_research::list_one_time_researches_by_user_id(pool.get_ref(), user_id).await {
        Ok(researches) => HttpResponse::Ok().json(researches),
        Err(e) => {
            log::error!("Failed to list one-time researches for user {user_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve research tasks.".into(),
            })
        }
    }
}