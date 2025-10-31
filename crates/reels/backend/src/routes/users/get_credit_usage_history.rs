//! Handler for retrieving user credit usage history
//!
//! This endpoint returns aggregated daily credit consumption data
//! for the authenticated user within a specified date range.
//! Supports optional organization filtering via x-organization-id header.

use crate::auth::tokens::Claims;
use crate::queries::credit_transactions::{get_credit_usage_history, CreditUsagePoint};
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Query parameters for credit usage history
#[derive(Debug, Deserialize, IntoParams)]
pub struct CreditUsageHistoryParams {
    /// Start date in format YYYY-MM-DD
    #[param(example = "2024-01-01")]
    pub start_date: String,
    
    /// End date in format YYYY-MM-DD
    #[param(example = "2024-01-31")]
    pub end_date: String,
    
    /// Comma-separated list of user IDs to filter (only used with organization context)
    #[param(example = "uuid1,uuid2")]
    #[serde(default)]
    pub user_ids: Option<String>,
}

/// Response containing credit usage history
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreditUsageHistoryResponse {
    /// List of daily credit usage data points
    pub data: Vec<CreditUsagePoint>,
}

/// Get credit usage history for the authenticated user
///
/// Returns aggregated daily credit consumption within the specified date range.
/// Only includes days where credits were actually consumed.
///
/// # Authentication
/// Requires valid JWT token in Authorization header
///
/// # Query Parameters
/// - `start_date`: Start date (inclusive) in YYYY-MM-DD format
/// - `end_date`: End date (inclusive) in YYYY-MM-DD format
///
/// # Headers
/// - `x-organization-id` (optional): Filter by specific organization. If not provided, shows personal usage.
/// 
/// # Query Parameters
/// - `user_ids` (optional): Comma-separated user IDs to filter (only used with organization context)
///
/// # Response
/// Returns array of credit usage points with date and credits consumed
///
/// # Errors
/// - 400: Invalid date format
/// - 401: Authentication required
/// - 500: Internal server error
#[utoipa::path(
    get,
    path = "/api/users/credit-usage-history",
    tag = "Users",
    params(
        CreditUsageHistoryParams,
        ("x-organization-id" = Option<String>, Header, description = "Optional organization ID to filter credit usage by organization")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Credit usage history retrieved successfully", body = Vec<CreditUsagePoint>),
        (status = 400, description = "Invalid date format", body = ErrorResponse),
        (status = 401, description = "Unauthorized - Invalid or missing JWT", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[get("/credit-usage-history")]
pub async fn get_credit_usage_history_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    params: web::Query<CreditUsageHistoryParams>,
) -> impl Responder {
    let user_id = claims.user_id;
    
    // Extract organization_id from header (optional)
    let organization_id = req
        .headers()
        .get("x-organization-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());
    
    // Parse optional user_ids filter
    let user_ids = if let Some(ids_str) = &params.user_ids {
        let parsed: Result<Vec<Uuid>, _> = ids_str
            .split(',')
            .map(|s| Uuid::parse_str(s.trim()))
            .collect();
        
        match parsed {
            Ok(ids) => Some(ids),
            Err(e) => {
                log::warn!("Invalid user_ids format: {}", e);
                return HttpResponse::BadRequest().json(ErrorResponse {
                    error: format!("Invalid user_ids format: {}", e),
                });
            }
        }
    } else {
        None
    };
    
    // Validate date format (basic check)
    if params.start_date.len() != 10 || params.end_date.len() != 10 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Invalid date format. Use YYYY-MM-DD".to_string(),
        });
    }
    
    match get_credit_usage_history(
        pool.get_ref(),
        user_id,
        &params.start_date,
        &params.end_date,
        organization_id,
        user_ids,
    )
    .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            log::error!("Failed to retrieve credit usage history for user {}: {:?}", user_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve credit usage history".to_string(),
            })
        }
    }
}

