//! User credit handlers
//!
//! This module provides API endpoints for user credit management including
//! daily credit claims and credit status retrieval.

use crate::auth::tokens::Claims;
use crate::queries::user_credit_allocation::{claim_daily_credits, get_user_credit_allocation_by_user_id};
use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::PgPool;

/// Response structure for daily credit claim
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct DailyCreditClaimResponse {
    /// Success message
    pub message: String,
    /// Current credits remaining after claim
    #[schema(example = "30.50", value_type = String)]
    pub credits_remaining: String,
    /// Daily credits that were claimed
    pub daily_credits_claimed: i32,
    /// Timestamp when credits were claimed
    pub claimed_at: chrono::DateTime<chrono::Utc>,
}

/// Error response structure for daily credit claim
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct DailyCreditClaimError {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
    /// Additional details
    pub details: Option<String>,
}

/// Claim daily credits for the authenticated user
///
/// This endpoint allows free plan users to claim their daily credits.
/// Users can only claim credits once per day.
///
/// # Authentication
/// Requires valid JWT token in Authorization header
///
/// # Response
/// Returns the updated credit allocation with success message
///
/// # Errors
/// - 400: User is not on free plan or already claimed today
/// - 401: Authentication required
/// - 404: User credit allocation not found
/// - 500: Internal server error
#[utoipa::path(
    post,
    path = "/api/users/claim-daily-credits",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Daily credits claimed successfully", body = DailyCreditClaimResponse),
        (status = 400, description = "Bad request - User not eligible for daily credits", body = DailyCreditClaimError),
        (status = 401, description = "Unauthorized - Invalid or missing JWT", body = DailyCreditClaimError),
        (status = 404, description = "Not found - User credit allocation not found", body = DailyCreditClaimError),
        (status = 500, description = "Internal server error", body = DailyCreditClaimError)
    )
)]
#[post("/claim-daily-credits")]
pub async fn claim_daily_credits_handler(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let user_id = claims.user_id;
    match claim_daily_credits(pool.get_ref(), user_id).await {
        Ok(credit_allocation) => {
            let response = DailyCreditClaimResponse {
                message: "Daily credits claimed successfully".to_string(),
                credits_remaining: credit_allocation.credits_remaining.to_string(),
                daily_credits_claimed: credit_allocation.daily_credits,
                claimed_at: credit_allocation.last_daily_credit_claimed_at
                    .unwrap_or_else(|| chrono::Utc::now()),
            };

            HttpResponse::Ok().json(response)
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = DailyCreditClaimError {
                error: "Credit allocation not found".to_string(),
                code: "CREDIT_ALLOCATION_NOT_FOUND".to_string(),
                details: Some("Please contact support to set up your credit allocation".to_string()),
            };
            HttpResponse::NotFound().json(error_response)
        }
        Err(sqlx::Error::Protocol(error_msg)) => {
            let error_response = DailyCreditClaimError {
                error: error_msg,
                code: "DAILY_CREDITS_NOT_AVAILABLE".to_string(),
                details: Some("Daily credits can only be claimed by free plan users once per day".to_string()),
            };
            HttpResponse::BadRequest().json(error_response)
        }
        Err(_e) => {
            let error_response = DailyCreditClaimError {
                error: "Internal server error".to_string(),
                code: "INTERNAL_SERVER_ERROR".to_string(),
                details: Some("Failed to process daily credit claim. Please try again later.".to_string()),
            };
            HttpResponse::InternalServerError().json(error_response)
        }
    }
}

/// Get user credits for the authenticated user
///
/// This endpoint provides information about the user's daily credit claim status
/// and when they can next claim daily credits.
///
/// # Authentication
/// Requires valid JWT token in Authorization header
///
/// # Response
/// Returns the user's credit allocation and claim status
#[utoipa::path(
    get,
    path = "/api/users/credits",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Daily credit status retrieved successfully", body = crate::db::user_credit_allocation::UserCreditAllocation),
        (status = 401, description = "Unauthorized - Invalid or missing JWT", body = DailyCreditClaimError),
        (status = 404, description = "Not found - User credit allocation not found", body = DailyCreditClaimError),
        (status = 500, description = "Internal server error", body = DailyCreditClaimError)
    )
)]
#[get("/credits")]
pub async fn get_user_credits_handler(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let user_id = claims.user_id;
    match get_user_credit_allocation_by_user_id(pool.get_ref(), user_id).await {
        Ok(Some(credit_allocation)) => {
            HttpResponse::Ok().json(credit_allocation)
        }
        Ok(None) => {
            let error_response = DailyCreditClaimError {
                error: "Credit allocation not found".to_string(),
                code: "CREDIT_ALLOCATION_NOT_FOUND".to_string(),
                details: Some("Please contact support to set up your credit allocation".to_string()),
            };
            HttpResponse::NotFound().json(error_response)
        }
        Err(_e) => {
            let error_response = DailyCreditClaimError {
                error: "Internal server error".to_string(),
                code: "INTERNAL_SERVER_ERROR".to_string(),
                details: Some("Failed to retrieve daily credit status. Please try again later.".to_string()),
            };
            HttpResponse::InternalServerError().json(error_response)
        }
    }
}
