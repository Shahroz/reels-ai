//! Contains helper function for verifying journey ownership.

use crate::routes::error_response::ErrorResponse;
use actix_web::HttpResponse;
use sqlx::PgPool;
use uuid::Uuid;

/// Checks if a user is the owner of a given Studio Journey.
/// Returns Ok(()) if owner, or an HttpResponse error otherwise.
pub async fn check_journey_ownership(
    pool: &PgPool,
    journey_id: Uuid,
    user_id: Uuid,
) -> Result<(), HttpResponse> {
    match sqlx::query_scalar!("SELECT user_id FROM studio_journeys WHERE id = $1", journey_id)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(owner_id)) => {
            if owner_id == user_id {
                Ok(())
            } else {
                log::warn!(
                    "Permission denied: User {} attempted to access journey {} owned by {}",
                    user_id,
                    journey_id,
                    owner_id
                );
                Err(HttpResponse::Forbidden().json(ErrorResponse {
                    error: "You do not have permission to access this journey.".to_string(),
                }))
            }
        }
        Ok(None) => Err(HttpResponse::NotFound().json(ErrorResponse {
            error: format!("Studio Journey not found: {}", journey_id),
        })),
        Err(e) => {
            log::error!("DB error checking journey ownership for journey {}: {}", journey_id, e);
            Err(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to verify journey ownership.".to_string(),
            }))
        }
    }
}