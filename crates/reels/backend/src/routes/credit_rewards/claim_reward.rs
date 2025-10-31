//! Claim credit reward handler
//!
//! This handler allows users to claim credit rewards when requirements are met.

use crate::auth::tokens::Claims;
use crate::queries::credit_rewards::claim_credit_reward;
use crate::routes::error_response::ErrorResponse;
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

/// Request to claim a credit reward
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ClaimRewardRequest {
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub reward_definition_id: Uuid,
}

/// Response for claiming a credit reward
#[derive(Debug, Serialize, ToSchema)]
pub struct ClaimRewardResponse {
    #[schema(example = "true")]
    pub success: bool,
    
    #[schema(example = "10")]
    pub credits_awarded: Option<i32>,
    
    #[schema(example = "Reward claimed successfully")]
    pub message: String,
}

/// Claim a credit reward for the authenticated user
#[utoipa::path(
    post,
    path = "/api/credit-rewards/claim",
    request_body = ClaimRewardRequest,
    responses(
        (status = 200, description = "Reward claimed successfully", body = ClaimRewardResponse),
        (status = 400, description = "Reward cannot be claimed", body = ClaimRewardResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security( // Add security requirement
        ("bearer_auth" = [])
    ),
    tag = "Credit Rewards"
)]
#[post("/claim")]
#[instrument(skip(pool, auth_claims, payload))]
pub async fn claim_reward(
    pool: web::Data<PgPool>,
    auth_claims: web::ReqData<Claims>,
    payload: web::Json<ClaimRewardRequest>,
) -> impl Responder {
    match claim_credit_reward(&pool, auth_claims.user_id, payload.reward_definition_id).await {
        Ok(Some(credits_awarded)) => HttpResponse::Ok().json(ClaimRewardResponse {
            success: true,
            credits_awarded: Some(credits_awarded),
            message: "Reward claimed successfully".to_string(),
        }),
        Ok(None) => HttpResponse::BadRequest().json(ClaimRewardResponse {
            success: false,
            credits_awarded: None,
            message: "Reward cannot be claimed. Requirements not met or already claimed".to_string(),
        }),
        Err(e) => {
            log::error!("Failed to claim credit reward: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to claim reward".to_string(),
            })
        }
    }
}
