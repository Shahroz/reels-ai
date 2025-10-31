//! Handler for fetching a single creative by ID.
//!
//! GET /api/creatives/{id}
//! Requires authentication and checks user ownership via the associated collection or sharing.

use crate::auth::tokens::Claims;
use crate::queries::creatives::get_creative_details::get_creative_details;
use crate::routes::creatives::responses::GetCreativeDetails;
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/creatives/{id}",
    responses(
        (status = 200, description = "Creative found", body = GetCreativeDetails),
        (status = 404, description = "Not found or not accessible", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    tag = "Creatives",
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::get("/{id}")]
#[instrument(skip(pool, claims))]
pub async fn get_creative_by_id(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    claims: Claims,
) -> impl Responder {
    let user_id = claims.user_id;
    let creative_id = *id;

    match get_creative_details(pool.get_ref(), user_id, creative_id).await {
        Ok(Some(details)) => HttpResponse::Ok().json(details),
        Ok(None) => {
            log::debug!(
                "Creative {creative_id} not found or not accessible for user {user_id}"
            );
            HttpResponse::NotFound().json(ErrorResponse::from("Creative not found or not accessible"))
        }
        Err(e) => {
            log::error!(
                "Failed to get creative details for creative {creative_id} for user {user_id}: {e}"
            );
            HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve creative"))
        }
    }
}
