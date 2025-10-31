//! Handler for a user deleting their own account.
// DELETE /api/users/me

use actix_web::{delete, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;

#[derive(serde::Deserialize)]
pub struct DeleteUserPath {
    // If we decide to allow admins to delete users by ID,
    // this might take a user_id. For now, it's /me, so no path param needed.
}

#[utoipa::path(
    delete,
    path = "/api/users/me",
    tag = "Users",
    responses(
        (status = 204, description = "User account deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User owns organizations"),
        (status = 404, description = "User not found"), // Should not happen if authenticated
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("user_auth" = [])
    )
)]
#[delete("/me")]
pub async fn delete_current_user_handler(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let user_id_to_delete = claims.user_id;

    // 1. Check if the user owns any non-personal organizations
    match check_owns_organizations(pool.get_ref(), user_id_to_delete).await {
        Ok(true) => {
            log::warn!("User {user_id_to_delete} attempted to delete account but owns non-personal organizations.");
            return HttpResponse::Forbidden().json(ErrorResponse {
                error: "Cannot delete account: You own one or more organizations. Please transfer ownership or delete them first.".to_string(),
            });
        }
        Ok(false) => {
            // Proceed to deletion
        }
        Err(e) => {
            log::error!("Failed to check organization ownership for user {user_id_to_delete}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to verify organization ownership before account deletion.".to_string(),
            });
        }
    }

    // 2. Delete the user from the database
    match crate::db::users::delete_user(pool.get_ref(), user_id_to_delete).await {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                log::warn!("User {user_id_to_delete} not found during deletion attempt.");
                return HttpResponse::NotFound().json(ErrorResponse {
                    error: "User not found.".to_string(),
                });
            }
            log::info!("User {user_id_to_delete} account deleted successfully.");
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            log::error!("Failed to delete user {user_id_to_delete}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to delete user account.".to_string(),
            })
        }
    }
}

/// Checks if a user owns any non-personal organizations.
/// Personal organizations don't block account deletion since they're meant to be user-specific.
async fn check_owns_organizations(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM organizations
        WHERE owner_user_id = $1 AND is_personal = false
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count.count.unwrap_or(0) > 0)
}


