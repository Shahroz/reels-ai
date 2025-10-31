//! Handler for retrieving the status of a Studio Journey share link.
use crate::auth::tokens::Claims;
use crate::queries::studio_journey_shares::get_share_by_journey_id::get_share_by_journey_id;
use crate::routes::error_response::ErrorResponse;
use crate::routes::studio_journey_shares::journey_ownership::check_journey_ownership;
use crate::routes::studio_journey_shares::responses::GetShareResponse;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/studio/journeys/{journey_id}/share",
    params(
        ("journey_id" = Uuid, Path, description = "ID of the Studio Journey to get share status for")
    ),
    responses(
        (status = 200, description = "Share link details found", body = GetShareResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User does not own this journey"),
        (status = 404, description = "Journey or share link not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Studio Journey Shares",
    security(("user_auth" = []))
)]
#[get("/{journey_id}/share")]
#[instrument(skip(pool, claims))]
pub async fn get_share(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let journey_id = path.into_inner();
    let user_id = claims.user_id;

    if let Err(response) = check_journey_ownership(&pool, journey_id, user_id).await {
        return response;
    }

    match get_share_by_journey_id(&pool, journey_id).await {
        Ok(Some(share)) => {
            HttpResponse::Ok().json(GetShareResponse {
                share_token: share.share_token,
                is_active: share.is_active,
            })
        }
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "No share link has been created for this journey.".to_string(),
        }),
        Err(e) => {
            log::error!("DB error getting journey share for journey {}: {}", journey_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve share status."))
        }
    }
}