//! Handler for fetching a single one-time research task by its ID.
//!
//! This endpoint allows a user to retrieve the status and details of a
//! one-time research task they have previously created.

use crate::queries;
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/one-time-researches/{id}",
    params(
        ("id" = Uuid, Path, description = "ID of the one-time research task to fetch")
    ),
    responses(
        (status = 200, description = "Details of the one-time research task", body = crate::db::one_time_research::OneTimeResearch),
        (status = 404, description = "Research task not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("jwt_token" = [])
    ),
    tag = "One-Time Research"
)]
#[actix_web::get("/{id}")]
#[tracing::instrument(skip(pool, auth, id))]
pub async fn get_one_time_research(
    pool: web::Data<PgPool>,
    auth: web::ReqData<crate::auth::tokens::Claims>,
    id: web::Path<Uuid>,
) -> impl Responder {
    let research_id = id.into_inner();

    match queries::one_time_research::get_one_time_research_by_id(
        pool.get_ref(),
        research_id,
        auth.user_id,
    )
    .await
    {
        Ok(research) => HttpResponse::Ok().json(research),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(ErrorResponse {
           error: format!("One-time research with id '{research_id}' not found."),
        }),
        Err(e) => {
            log::error!(
                "Failed to fetch one-time research {} for user {}: {}",
                research_id,
                auth.user_id,
                e
            );
            HttpResponse::InternalServerError().json(ErrorResponse {
               error: "Failed to retrieve research task.".to_string(),
            })
        }
    }
}