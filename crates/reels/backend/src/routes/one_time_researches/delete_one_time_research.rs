//! Handler for deleting a one-time research task.
//!
//! This endpoint allows a user to delete their own one-time research task by its ID.
//! It ensures that a user can only delete tasks they own.
//! Follows the one-item-per-file guideline.

use actix_web::{web, Responder};
use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;



#[utoipa::path(
    delete,
    path = "/api/one-time-researches/{id}",
    params(
        ("id" = uuid::Uuid, Path, description = "ID of the one-time research to delete")
    ),
    responses(
        (status = 204, description = "Research task deleted successfully"),
        (status = 404, description = "Research task not found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("jwt_token" = [])
    ),
    tag = "One-Time Research"
)]
#[actix_web::delete("/{id}")]
#[tracing::instrument(skip(pool, path, auth))]
pub async fn delete_one_time_research(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<uuid::Uuid>,
    auth: Claims,
) -> impl Responder {
    let research_id = path.into_inner();
    let user_id = auth.user_id;

    match crate::queries::one_time_research::delete_one_time_research::delete_one_time_research(
        &pool,
        research_id,
        user_id,
    )
    .await
    {
        Ok(rows_affected) if rows_affected > 0 => actix_web::HttpResponse::NoContent().finish(),
        Ok(_) => actix_web::HttpResponse::NotFound().json(ErrorResponse {
            error: "Research task not found or you do not have permission to delete it."
                .to_string(),
        }),
        Err(e) => {
            tracing::error!("Failed to delete one-time research from DB: {:?}", e);
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Internal server error".to_string(),
            })
        }
    }
}