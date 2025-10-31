//! Handler for activating a user in the admin panel.

use crate::auth::tokens::Claims;
use crate::db::users;
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse, Responder};
use sqlx::{types::Uuid, PgPool};
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/api/admin/users/{user_id}/activate",
    tag = "Admin",
    params(
        ("user_id" = Uuid, Path, description = "The ID of the user to activate.")
    ),
    responses(
        (status = 204, description = "Successfully activated user"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/{user_id}/activate")]
#[instrument(skip(pool, auth_claims))]
pub async fn activate_user_handler(
    pool: web::Data<PgPool>,
    auth_claims: Claims,
    user_id: web::Path<Uuid>,
) -> impl Responder {
    let user_id = user_id.into_inner();

    match users::set_user_status(&pool, user_id, "active").await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(ErrorResponse {
            error: "User not found".to_string(),
        }),
        Err(e) => {
            log::error!("Failed to activate user: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to activate user.".to_string(),
            })
        }
    }
}