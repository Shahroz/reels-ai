//! Handler for fetching data for a publicly shared journey.
use crate::queries::studio_journey_shares::get_journey_by_share_token::get_journey_by_share_token;
use crate::queries::studio_journeys::get_public_journey_view::get_public_journey_view;
use crate::routes::error_response::ErrorResponse;
use crate::routes::public::responses::get_public_journey_response::{
    GetPublicJourneyResponse, PublicJourneyNodeResponse,
};
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/public/journeys/view/{share_token}",
    params(
        ("share_token" = Uuid, Path, description = "The secret token for the shared journey")
    ),
    responses(
        (status = 200, description = "Public journey data found", body = GetPublicJourneyResponse),
        (status = 404, description = "Share token is invalid or inactive", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Public Journeys"
)]
#[get("/view/{share_token}")]
#[instrument(skip(pool))]
pub async fn view_journey(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let share_token = path.into_inner();

    // 1. Find the active share record by its token.
    let share = match get_journey_by_share_token(&pool, share_token).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Share link not found or is no longer active.".to_string(),
            });
        }
        Err(e) => {
            log::error!("DB error getting journey share by token {}: {}", share_token, e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::from("Failed to retrieve share link."));
        }
    };

    // 2. Fetch the public view data for the associated journey.
    match get_public_journey_view(&pool, share.journey_id).await {
        Ok(Some(journey_view)) => {
            let response = GetPublicJourneyResponse {
                name: journey_view.name,
                nodes: journey_view
                    .nodes
                    .into_iter()
                    .map(|node| PublicJourneyNodeResponse {
                        id: node.id,
                        name: node.name,
                        url: node.url,
                        parent_id: node.parent_node_id,
                    })
                    .collect(),
            };
            HttpResponse::Ok().json(response)
        }
        Ok(None) => {
            // This case is unlikely if a share record exists, but handle it defensively.
            HttpResponse::NotFound().json(ErrorResponse {
                error: "The shared journey could not be found.".to_string(),
            })
        }
        Err(e) => {
            log::error!("DB error getting public journey view for journey {}: {}", share.journey_id, e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::from("Failed to retrieve journey data."))
        }
    }
}