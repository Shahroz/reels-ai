//! Handler for updating a user in the admin panel.

use crate::auth::tokens::Claims;
use crate::db::users::{self, PublicUser};
use crate::routes::admin::users::update_user_request::UpdateUserRequest;
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse, Responder};
use sqlx::{types::Uuid, PgPool};
use tracing::instrument;

#[utoipa::path(
    put,
    path = "/api/admin/users/{user_id}",
    tag = "Admin",
    params(
        ("user_id" = Uuid, Path, description = "The ID of the user to update.")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Successfully updated user", body = PublicUser),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::put("/{user_id}")]
#[instrument(skip(pool, auth_claims, payload))]
pub async fn update_user_handler(
    pool: web::Data<PgPool>,
    auth_claims: Claims,
    user_id: web::Path<Uuid>,
    payload: web::Json<UpdateUserRequest>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    let UpdateUserRequest {
        is_admin,
        status,
        feature_flags,
    } = payload.into_inner();

    match users::admin_update_user(&pool, user_id, is_admin, &status, &feature_flags).await {
        Ok(user) => HttpResponse::Ok().json(PublicUser::from(user)),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(ErrorResponse {
            error: "User not found".to_string(),
        }),
        Err(e) => {
            log::error!("Failed to update user: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to update user.".to_string(),
            })
        }
    }
}