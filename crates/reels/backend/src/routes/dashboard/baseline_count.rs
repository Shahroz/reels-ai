//! Handler for providing baseline entity counts for cumulative charts.
//!
//! Defines the GET `/api/dashboard/baseline-count` endpoint.
//! This endpoint returns the total count of entities that existed before a specified date,
//! which is used as the starting point for cumulative chart calculations.
//!
//! Revision History
//! - 2025-10-06 @AI: Initial implementation of baseline count endpoint for cumulative charts.

use crate::routes::error_response::ErrorResponse;
use crate::queries::dashboard::query_baseline_count::query_baseline_count;
use crate::routes::dashboard::daily_activity_stats::{ActivityEntityType, ActivityTimePeriod, calculate_datetime_range};

/// Query parameters for baseline count endpoint.
#[derive(serde::Deserialize, Debug, utoipa::IntoParams, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BaselineCountParams {
    /// Type of entity to get baseline count for
    pub entity_type: ActivityEntityType,
    /// Predefined time period (optional, mutually exclusive with start_date/end_date)
    pub time_period: Option<ActivityTimePeriod>,
    /// Custom start date (optional, ISO 8601 format)
    #[param(value_type = Option<String>)]
    pub start_date: Option<chrono::NaiveDate>,
    /// Custom end date (optional, ISO 8601 format)
    #[param(value_type = Option<String>)]
    pub end_date: Option<chrono::NaiveDate>,
    /// Filter by user email (admin only)
    pub user_email: Option<String>,
}

/// Response for baseline count endpoint.
#[derive(serde::Serialize, Debug, utoipa::ToSchema)]
pub struct BaselineCountResponse {
    /// The total count of entities that existed before the start date
    #[schema(example = 1523)]
    pub baseline_count: i64,
    /// The start date used for the baseline calculation
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
    pub start_date: chrono::DateTime<chrono::Utc>,
    /// The entity type queried
    pub entity_type: String,
}

#[utoipa::path(
    get,
    path = "/api/dashboard/baseline-count",
    tag = "Dashboard",
    params(BaselineCountParams),
    security(
        ("user_auth" = [])
    ),
    responses(
        (status = 200, description = "Baseline count retrieved successfully", body = BaselineCountResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden (e.g. non-admin requesting specific user email)", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[actix_web::get("/baseline-count")]
pub async fn get_baseline_count(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    params: actix_web::web::Query<BaselineCountParams>,
) -> impl actix_web::Responder {
    // Authorization: only admins can filter by user email
    if params.user_email.is_some() && !claims.is_admin {
        return actix_web::HttpResponse::Forbidden().json(ErrorResponse {
            error: std::string::String::from("Admin access required to filter by user email."),
        });
    }

    // Calculate the date range using the same logic as daily_activity_stats
    let (start_datetime, _end_datetime) = calculate_datetime_range(
        params.start_date,
        params.end_date,
        params.time_period,
        chrono::Utc::now(),
    );

    // Query the baseline count (entities created before start_datetime)
    let result = query_baseline_count(
        pool.get_ref(),
        params.entity_type,
        start_datetime,
        params.user_email.clone(),
    )
    .await;

    match result {
        Ok(count) => {
            let response = BaselineCountResponse {
                baseline_count: count,
                start_date: start_datetime,
                entity_type: params.entity_type.to_string(),
            };
            actix_web::HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Failed to fetch baseline count: {:?}", e);
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to fetch baseline count: {}", e),
            })
        }
    }
}

