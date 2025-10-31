//! Handler for deleting a user favorite.
use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;
use actix_web::{delete, web, HttpResponse};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/api/user-favorites/{favorite_id}",
    responses(
        (status = 204, description = "Favorite deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - User cannot delete this favorite", body = ErrorResponse),
        (status = 404, description = "Favorite not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "User Favorites",
    security(("user_auth" = []))
)]
#[delete("/{favorite_id}")]
#[instrument(skip(pool, claims))]
pub async fn delete_favorite(
    pool: web::Data<PgPool>,
    favorite_id: web::Path<Uuid>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, actix_web::Error> {
    let authenticated_user_id = claims.user_id;
    let favorite_id = *favorite_id;

    log::info!("Received delete_favorite request. User ID: {}, Favorite ID: {}", authenticated_user_id, favorite_id);

    // Check if favorite exists and belongs to the user
    let favorite = sqlx::query!(
        "SELECT id, user_id FROM user_favorites WHERE id = $1",
        favorite_id
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|e| {
        log::error!("DB error checking favorite existence: {e}");
        actix_web::error::ErrorInternalServerError("Failed to check favorite existence")
    })?;

    match favorite {
        Some(fav) => {
            if fav.user_id != authenticated_user_id {
                log::warn!("User {} attempted to delete favorite {} belonging to user {}",
                          authenticated_user_id, favorite_id, fav.user_id);
                return Ok(HttpResponse::Forbidden().json(ErrorResponse::from("You can only delete your own favorites")));
            }
        }
        None => {
            log::warn!("Favorite {favorite_id} not found for deletion by user {authenticated_user_id}");
            return Ok(HttpResponse::NotFound().json(ErrorResponse::from("Favorite not found")));
        }
    }

    // Delete the favorite
    let deleted_count = sqlx::query!(
        "DELETE FROM user_favorites WHERE id = $1 AND user_id = $2",
        favorite_id,
        authenticated_user_id
    )
    .execute(&**pool)
    .await
    .map_err(|e| {
        log::error!("DB error deleting favorite: {e}");
        actix_web::error::ErrorInternalServerError("Failed to delete favorite")
    })?
    .rows_affected();

    if deleted_count == 0 {
        log::warn!("No favorite deleted for ID {favorite_id} and user {authenticated_user_id}");
        return Ok(HttpResponse::NotFound().json(ErrorResponse::from("Favorite not found")));
    }

    log::info!("Successfully deleted favorite {} for user {}", favorite_id, authenticated_user_id);

    Ok(HttpResponse::NoContent().finish())
}