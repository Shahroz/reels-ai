//! Get available credit reward definitions handler
//!
//! This handler retrieves all available credit reward definitions.

use crate::queries::credit_rewards::get_active_reward_definitions;
use crate::routes::error_response::ErrorResponse;
use crate::db::credit_rewards::CreditRewardDefinition;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;

/// Get all active credit reward definitions
#[utoipa::path(
    get,
    path = "/api/credit-rewards",
    responses(
        (status = 200, description = "Successfully retrieved reward definitions", body = Vec<CreditRewardDefinition>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security( // Add security requirement
        ("bearer_auth" = [])
    ),
    tag = "Credit Rewards"
)]
#[get("")]
#[instrument(skip(pool))]
pub async fn get_reward_definitions(
    pool: web::Data<PgPool>,
) -> impl Responder {
    match get_active_reward_definitions(&pool).await {
        Ok(definitions) => HttpResponse::Ok().json(definitions),
        Err(e) => {
            log::error!("Failed to get reward definitions: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve reward definitions".to_string(),
            })
        }
    }
}
