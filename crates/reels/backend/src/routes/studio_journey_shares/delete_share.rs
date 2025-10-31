//! Handler for deactivating a Studio Journey share link.
use crate::auth::tokens::Claims;
use crate::queries::studio_journey_shares::deactivate_share::deactivate_share;
use crate::routes::error_response::ErrorResponse;
use crate::routes::studio_journey_shares::journey_ownership::check_journey_ownership;
use actix_web::{delete, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/api/studio/journeys/{journey_id}/share",
    operation_id = "delete_journey_share",
    params(
        ("journey_id" = Uuid, Path, description = "ID of the Studio Journey to deactivate sharing for")
    ),
    responses(
        (status = 204, description = "Share link deactivated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User does not own this journey"),
        (status = 404, description = "Journey not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Studio Journey Shares",
    security(("user_auth" = []))
)]
#[delete("/{journey_id}/share")]
#[instrument(skip(pool, claims))]
pub async fn delete_share(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let journey_id = path.into_inner();
    let user_id = claims.user_id;

    if let Err(response) = check_journey_ownership(&pool, journey_id, user_id).await {
        return response;
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to begin transaction to deactivate journey share: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Database error."));
        }
    };

    match deactivate_share(&mut tx, journey_id).await {
        Ok(rows_affected) => {
            if let Err(e) = tx.commit().await {
                log::error!("Failed to commit transaction for journey share deactivation: {}", e);
                return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to finalize share deactivation."));
            }
            if rows_affected == 0 {
                // This could happen if a share never existed.
                // Returning 204 is fine as the state is "not active".
                log::info!("Deactivate share called for journey {}, but no active share was found to deactivate.", journey_id);
            }
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            let _ = tx.rollback().await;
            log::error!("Failed to deactivate journey share for journey {}: {}", journey_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to deactivate share link."))
        }
    }
}