//! Get user credit rewards handler
//!
//! This handler retrieves all credit rewards for the authenticated user.

use crate::auth::tokens::Claims;
use crate::queries::credit_rewards::get_user_credit_rewards as get_user_credit_rewards_query;
use crate::routes::error_response::ErrorResponse;
use crate::db::credit_rewards::UserCreditRewardWithDefinition;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;

/// Get all credit rewards for the authenticated user
#[utoipa::path(
    get,
    path = "/api/credit-rewards/my-rewards",
    responses(
        (status = 200, description = "Successfully retrieved user credit rewards", body = Vec<UserCreditRewardWithDefinition>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security( // Add security requirement
        ("bearer_auth" = [])
    ),
    tag = "Credit Rewards"
)]
#[get("/my-rewards")]
#[instrument(skip(pool, auth_claims))]
pub async fn get_user_credit_rewards(
    pool: web::Data<PgPool>,
    auth_claims: web::ReqData<Claims>,
) -> impl Responder {
    match get_user_credit_rewards_query(&pool, auth_claims.user_id).await {
        Ok(rewards) => HttpResponse::Ok().json(rewards),
        Err(e) => {
            log::error!("Failed to get user credit rewards: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve credit rewards".to_string(),
            })
        }
    }
}
