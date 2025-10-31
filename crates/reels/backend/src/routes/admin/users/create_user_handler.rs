//! Handler for creating a new user via the admin panel.
//!
//! This endpoint allows administrators to create new user accounts with specified email,
//! password, and admin status. Performs password hashing before storage and creates the
//! complete user record with appropriate defaults.
//!
//! Revision History:
//! - 2025-10-10: Removed manual admin check (now handled by AdminGuard middleware).

use crate::auth::tokens::Claims;
use crate::db::users::{self, PublicUser};
use crate::routes::admin::users::create_user_request::CreateUserRequest;
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/api/admin/users",
    tag = "Admin",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "Successfully created user", body = PublicUser),
        (status = 400, description = "Bad Request, e.g., invalid input or user already exists", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("")]
#[instrument(skip(pool, auth_claims, payload))]
pub async fn create_user_handler(
    pool: web::Data<PgPool>,
    auth_claims: Claims,
    payload: web::Json<CreateUserRequest>,
) -> impl Responder {
    // Hash the password
    let password_hash = match bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            log::error!("Failed to hash password: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to process user credentials.".to_string(),
            });
        }
    };

    match users::admin_create_user(
        &pool,
        &payload.email,
        &password_hash,
        payload.is_admin,
        &payload.status,
        &payload.feature_flags,
    )
    .await
    {
        Ok(user) => {
            let public_user: PublicUser = user.into();
            HttpResponse::Created().json(public_user)
        }
        Err(e) => {
            // Check for unique constraint violation (user already exists)
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return HttpResponse::BadRequest().json(ErrorResponse {
                        error: "A user with this email already exists.".to_string(),
                    });
                }
            }
            log::error!("Failed to create user in database: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create user.".to_string(),
            })
        }
    }
}