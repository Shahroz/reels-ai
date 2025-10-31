//! Handler for deactivating a user in the admin panel.

use crate::auth::tokens::Claims;
use crate::db::users;
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse, Responder};
use sqlx::{types::Uuid, PgPool};
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/api/admin/users/{user_id}/deactivate",
    tag = "Admin",
    params(
        ("user_id" = Uuid, Path, description = "The ID of the user to deactivate.")
    ),
    responses(
        (status = 204, description = "Successfully deactivated user"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/{user_id}/deactivate")]
#[instrument(skip(pool, auth_claims))]
pub async fn deactivate_user_handler(
    pool: web::Data<PgPool>,
    auth_claims: Claims,
    user_id: web::Path<Uuid>,
) -> impl Responder {
   let user_id = user_id.into_inner();

   match users::set_user_status(&pool, user_id, "deactivated").await {
       Ok(_) => HttpResponse::NoContent().finish(),
       Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(ErrorResponse {
           error: "User not found".to_string(),
        }),
        Err(e) => {
            log::error!("Failed to deactivate user: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to deactivate user.".to_string(),
            })
        }
    }
}
