//! HTTP handler for action type breakdown endpoint.
//!
//! Returns credit usage breakdown grouped by action type with optional
//! organization and user filtering via headers and query params.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Added revision history (lighter-weight approach: keeping use statements)

use crate::auth::tokens::Claims;
use crate::queries::credit_transactions::get_action_type_breakdown;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(serde::Deserialize, ToSchema)]
pub struct GetActionTypeBreakdownParams {
    #[schema(example = "2024-01-01")]
    pub start_date: String,
    #[schema(example = "2024-01-31")]
    pub end_date: String,
    #[schema(example = "uuid1,uuid2")]
    #[serde(default)]
    pub user_ids: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/users/action-type-breakdown",
    tag = "Users",
    params(
        ("start_date" = String, Query, description = "Start date for action type breakdown (YYYY-MM-DD)"),
        ("end_date" = String, Query, description = "End date for action type breakdown (YYYY-MM-DD)"),
        ("user_ids" = Option<String>, Query, description = "Comma-separated user IDs to filter (only used with organization context)"),
        ("x-organization-id" = Option<String>, Header, description = "Optional organization ID to filter credit usage by organization")
    ),
    responses(
        (status = 200, description = "Action type breakdown retrieved successfully", body = Vec<crate::queries::credit_transactions::ActionTypeBreakdown>),
        (status = 400, description = "Invalid date format", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized - Invalid or missing JWT", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    )
)]
#[get("/action-type-breakdown")]
pub async fn get_action_type_breakdown_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    params: web::Query<GetActionTypeBreakdownParams>,
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
                return HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse {
                    error: format!("Invalid user_ids format: {}", e),
                });
            }
        }
    } else {
        None
    };
    
    match get_action_type_breakdown(pool.get_ref(), user_id, &params.start_date, &params.end_date, organization_id, user_ids).await {
        Ok(breakdown) => HttpResponse::Ok().json(breakdown),
        Err(e) => {
            log::error!("Failed to retrieve action type breakdown for user {}: {:?}", user_id, e);
            HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: format!("Failed to retrieve action type breakdown: {}", e),
            })
        }
    }
}

