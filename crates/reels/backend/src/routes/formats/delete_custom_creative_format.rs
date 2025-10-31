//! Handler for deleting a custom creative format.
//!
//! DELETE /api/custom-creative-formats/{id}

//! Allows authenticated users to delete their own custom formats.
//! Requires JWT authentication and ownership check.
//! Admin users can delete any format regardless of ownership.
//! Returns No Content on success.

use crate::queries::custom_creative_formats::{delete, exists};
use crate::routes::error_response::ErrorResponse;
use tracing::instrument;

#[utoipa::path(
    delete,
    path = "/api/formats/custom-creative-formats/{id}",
    tag = "Formats",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "ID of the custom format to delete")
    ),
    responses(
        (status = 204, description = "Custom format deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not owner (admin users can delete any format)"),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[actix_web::delete("/{id}")]
#[instrument(skip(pool, claims))]
pub async fn delete_custom_creative_format(
    pool: actix_web::web::Data<sqlx::PgPool>,
    id: actix_web::web::Path<uuid::Uuid>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let user_id: uuid::Uuid = claims.user_id;
    let is_admin: bool = claims.is_admin;
    let format_id = *id;

    match delete::delete(pool.get_ref(), format_id, user_id, is_admin).await {
        Ok(rows_affected) => {
            if rows_affected == 1 {
                actix_web::HttpResponse::NoContent().finish()
            } else {
                // If 0 rows affected, the format either doesn't exist or isn't owned by the user.
                // We check if it exists at all to return a 403 vs 404.
                match exists::exists(pool.get_ref(), format_id).await {
                    Ok(true) => {
                        // Format exists, but we couldn't delete it. Forbidden (for non-admin users).
                        actix_web::HttpResponse::Forbidden().json(ErrorResponse {
                            error: "Forbidden: You do not own this format.".to_string(),
                        })
                    }
                    Ok(false) => {
                        // Format does not exist. Not Found.
                        actix_web::HttpResponse::NotFound().json(ErrorResponse {
                            error: "Custom format not found.".to_string(),
                        })
                    }
                    Err(e) => {
                        eprintln!("Database error during existence check for format {format_id}: {e:?}");
                        actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                            error: "Internal server error.".to_string(),
                        })
                    }
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Database error deleting custom creative format {format_id} for user {user_id}: {e:?}"
            );
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to delete custom format.".to_string(),
            })
        }
    }
}

#[cfg(test)] // Changed from FALSE
mod tests {
    // Tests need database and auth setup.
    // #[sqlx::test]
    // async fn test_delete_custom_format_success(pool: sqlx::PgPool) {
    //     // Setup user, token, create a format owned by user...
    //     // Make DELETE request with auth and path ID...
    //     // Assert 204 No Content...
    //     // Verify format is deleted from database...
    // }
    //
    // #[sqlx::test]
    // async fn test_delete_custom_format_not_owner(pool: sqlx::PgPool) {
    //     // Setup user1, token1, user2, create format owned by user2...
    //     // Make DELETE request AS USER1 to user2's format ID...
    //     // Assert 403 Forbidden...
    //     // Verify format still exists...
    // }
    //
    // #[sqlx::test]
    // async fn test_delete_custom_format_not_found(pool: sqlx::PgPool) {
    //     // Setup user, token...
    //     // Make DELETE request with auth to a non-existent UUID...
    //     // Assert 404 Not Found...
    // }
}