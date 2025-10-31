//! Handler for providing custom events statistics from analytics_events table
//!
//! Defines the GET `/api/dashboard/custom-events-stats` endpoint.
//! Returns aggregated event counts and unique user counts grouped by event_name,
//! formatted for chart rendering with dual series (total events & unique users).
//!
//! Revision History
//! - 2025-09-16T00:00:00Z @AI: Initial implementation of custom events statistics endpoint.

use crate::routes::dashboard::chart_models::ChartData;
use crate::routes::dashboard::series_data_point::SeriesDataPoint;
use crate::routes::error_response::ErrorResponse;
use crate::routes::dashboard::daily_activity_stats::{ActivityTimePeriod, calculate_datetime_range};
use crate::queries::dashboard::query_custom_events_stats::query_custom_events_stats;
use crate::queries::dashboard::query_custom_events_daily_stats::query_custom_events_daily_stats;
use crate::queries::dashboard::query_cohort_registration_analysis::{
    query_cohort_registration_comprehensive, CohortRegistrationStat
};

/// Parameters for requesting custom events statistics
#[derive(serde::Deserialize, Debug, utoipa::ToSchema, utoipa::IntoParams)]
pub struct CustomEventsStatsParams {
    /// Optional start date for the statistics range (YYYY-MM-DD)
    #[schema(value_type = Option<String>, format = "date", example = "2023-01-01", nullable = true)]
    pub start_date: Option<chrono::NaiveDate>,
    /// Optional end date for the statistics range (YYYY-MM-DD)  
    #[schema(value_type = Option<String>, format = "date", example = "2023-01-31", nullable = true)]
    pub end_date: Option<chrono::NaiveDate>,
    /// Optional time period (e.g., "last_7_days", "last_30_days", "last_90_days", "this_month", "last_month")
    /// If provided, this can override/calculate start/end dates. Defaults to Last30Days if not specified.
    #[schema(example = "last_30_days")]
    pub time_period: Option<ActivityTimePeriod>,
}

/// Parameters for requesting daily breakdown of custom events for a specific event type
#[derive(serde::Deserialize, Debug, utoipa::ToSchema, utoipa::IntoParams)]
pub struct CustomEventsDailyStatsParams {
    /// Event name to get daily breakdown for
    #[schema(example = "user_login_successful")]
    pub event_name: String,
    /// Optional start date for the statistics range (YYYY-MM-DD)
    #[schema(value_type = Option<String>, format = "date", example = "2023-01-01", nullable = true)]
    pub start_date: Option<chrono::NaiveDate>,
    /// Optional end date for the statistics range (YYYY-MM-DD)  
    #[schema(value_type = Option<String>, format = "date", example = "2023-01-31", nullable = true)]
    pub end_date: Option<chrono::NaiveDate>,
    /// Optional time period (reuse ActivityTimePeriod enum)
    #[schema(example = "last_30_days")]
    pub time_period: Option<ActivityTimePeriod>,
}

/// Parameters for requesting cohort registration analysis
#[derive(serde::Deserialize, Debug, utoipa::ToSchema, utoipa::IntoParams)]
pub struct CohortRegistrationAnalysisParams {
    /// Start date for user cohort based on registration (YYYY-MM-DD)
    #[schema(value_type = String, format = "date", example = "2023-01-01")]
    pub cohort_start_date: chrono::NaiveDate,
    /// End date for user cohort based on registration (YYYY-MM-DD)
    #[schema(value_type = String, format = "date", example = "2023-01-31")]
    pub cohort_end_date: chrono::NaiveDate,
    /// Optional start date for the events analysis range (YYYY-MM-DD)
    #[schema(value_type = Option<String>, format = "date", example = "2023-01-01", nullable = true)]
    pub event_start_date: Option<chrono::NaiveDate>,
    /// Optional end date for the events analysis range (YYYY-MM-DD)  
    #[schema(value_type = Option<String>, format = "date", example = "2023-01-31", nullable = true)]
    pub event_end_date: Option<chrono::NaiveDate>,
    /// Optional time period for events analysis (defaults to all time if not specified)
    #[schema(example = "last_30_days")]
    pub time_period: Option<ActivityTimePeriod>,
}

/// Response structure for cohort registration analysis
#[derive(serde::Serialize, Debug, utoipa::ToSchema)]
pub struct CohortRegistrationAnalysisResponse {
    /// Title describing the analysis
    #[schema(example = "User Registration Cohort Analysis")]
    pub title: String,
    /// Cohort date range description
    #[schema(example = "Users registered between 2023-01-01 and 2023-01-31")]
    pub cohort_description: String,
    /// Event date range description
    #[schema(example = "Events from 2023-01-01 to 2023-01-31")]
    pub event_description: String,
    /// Analysis results grouped by dimension
    pub dimensions: std::collections::HashMap<String, std::vec::Vec<CohortDimensionStat>>,
}

/// Statistics for a specific dimension value in cohort analysis
#[derive(serde::Serialize, Debug, utoipa::ToSchema)]
pub struct CohortDimensionStat {
    /// The value for this dimension (e.g., "Chrome", "Mobile", "Windows")
    #[schema(example = "Chrome")]
    pub value: String,
    /// Total events for this dimension value
    #[schema(example = 150)]
    pub total_events: i64,
    /// Unique users for this dimension value
    #[schema(example = 85)]
    pub unique_users: i64,
    /// Percentage of total events for this dimension
    #[schema(example = 42.5)]
    pub percentage: f64,
}

#[utoipa::path(
    get,
    path = "/api/dashboard/custom-events-stats",
    tag = "Dashboard",
    params(CustomEventsStatsParams),
    security(("user_auth" = [])),
    responses(
        (status = 200, description = "Custom events statistics retrieved successfully", body = ChartData),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse), 
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
#[actix_web::get("/custom-events-stats")]
pub async fn get_custom_events_stats(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    params: actix_web::web::Query<CustomEventsStatsParams>,
) -> impl actix_web::Responder {
    // Restrict access to admin users only
    if !claims.is_admin {
        return actix_web::HttpResponse::Forbidden().json(ErrorResponse {
            error: std::string::String::from("Admin access required to view custom events statistics."),
        });
    }

    let (start_datetime, end_datetime) = calculate_datetime_range(
        params.start_date,
        params.end_date,
        params.time_period,
        chrono::Utc::now(),
    );

    let result = query_custom_events_stats(
        pool.get_ref(),
        start_datetime,
        end_datetime,
    ).await;

    match result {
        Ok(custom_event_stats) => {
            if custom_event_stats.is_empty() {
                // Return empty chart data when no events found
                let chart_data = ChartData {
                    title: Some(std::string::String::from("Custom Events Statistics")),
                    labels: std::vec::Vec::new(),
                    series: std::vec::Vec::new(),
                };
                return actix_web::HttpResponse::Ok().json(chart_data);
            }

            let mut labels: std::vec::Vec<std::string::String> = std::vec::Vec::new();
            let mut total_events_data: std::vec::Vec<SeriesDataPoint> = std::vec::Vec::new();
            let mut unique_users_data: std::vec::Vec<SeriesDataPoint> = std::vec::Vec::new();

            for stat in custom_event_stats {
                labels.push(stat.event_name.clone());
                total_events_data.push(SeriesDataPoint { value: stat.total_events.into() });
                unique_users_data.push(SeriesDataPoint { value: stat.unique_users.into() });
            }

            let total_events_series = crate::routes::dashboard::chart_models::ChartSeries {
                name: std::string::String::from("Total Events"),
                data: total_events_data,
            };

            let unique_users_series = crate::routes::dashboard::chart_models::ChartSeries {
                name: std::string::String::from("Unique Users"),
                data: unique_users_data,
            };

            let chart_data = ChartData {
                title: Some(std::string::String::from("Custom Events Statistics")),
                labels,
                series: vec![total_events_series, unique_users_series],
            };
            
            actix_web::HttpResponse::Ok().json(chart_data)
        }
        Err(e) => {
            log::error!("Failed to fetch custom events statistics: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: std::string::String::from("Failed to retrieve custom events statistics."),
            })
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/dashboard/custom-events-daily-stats",
    tag = "Dashboard",
    params(CustomEventsDailyStatsParams),
    security(
        ("user_auth" = [])
    ),
    responses(
        (status = 200, description = "Daily custom events statistics retrieved successfully", body = ChartData),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
#[actix_web::get("/custom-events-daily-stats")]
pub async fn get_custom_events_daily_stats(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    params: actix_web::web::Query<CustomEventsDailyStatsParams>,
) -> impl actix_web::Responder {
    if !claims.is_admin {
        return actix_web::HttpResponse::Forbidden().json(ErrorResponse {
            error: std::string::String::from("Admin access required."),
        });
    }

    if params.event_name.trim().is_empty() {
        return actix_web::HttpResponse::BadRequest().json(ErrorResponse {
            error: std::string::String::from("Event name cannot be empty."),
        });
    }

    let (start_datetime, end_datetime) = calculate_datetime_range(
        params.start_date,
        params.end_date,
        params.time_period,
        chrono::Utc::now(),
    );

    let result = query_custom_events_daily_stats(
        pool.get_ref(),
        &params.event_name,
        start_datetime,
        end_datetime,
    )
    .await;

    match result {
        Ok(daily_stats) => {
            let mut labels: std::vec::Vec<std::string::String> = std::vec::Vec::new();
            let mut total_events_data: std::vec::Vec<SeriesDataPoint> = std::vec::Vec::new();
            let mut unique_users_data: std::vec::Vec<SeriesDataPoint> = std::vec::Vec::new();

            for stat in daily_stats {
                labels.push(stat.activity_date.format("%Y-%m-%d").to_string());
                total_events_data.push(SeriesDataPoint { value: stat.total_events.into() });
                unique_users_data.push(SeriesDataPoint { value: stat.unique_users.into() });
            }

            let total_events_series = crate::routes::dashboard::chart_models::ChartSeries {
                name: format!("Total {} Events", params.event_name),
                data: total_events_data,
            };

            let unique_users_series = crate::routes::dashboard::chart_models::ChartSeries {
                name: format!("Unique Users ({})", params.event_name),
                data: unique_users_data,
            };

            let chart_data = ChartData {
                title: Some(format!("Daily Breakdown: {}", params.event_name)),
                labels,
                series: vec![total_events_series, unique_users_series],
            };
            
            actix_web::HttpResponse::Ok().json(chart_data)
        }
        Err(e) => {
            log::error!("Failed to fetch daily custom events statistics: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: std::string::String::from("Failed to retrieve daily custom events statistics."),
            })
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/dashboard/cohort-registration-analysis",
    tag = "Dashboard",
    params(CohortRegistrationAnalysisParams),
    security(("user_auth" = [])),
    responses(
        (status = 200, description = "Cohort registration analysis retrieved successfully", body = CohortRegistrationAnalysisResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
#[actix_web::get("/cohort-registration-analysis")]
pub async fn get_cohort_registration_analysis(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    params: actix_web::web::Query<CohortRegistrationAnalysisParams>,
) -> impl actix_web::Responder {
    if !claims.is_admin {
        return actix_web::HttpResponse::Forbidden().json(ErrorResponse {
            error: std::string::String::from("Admin access required."),
        });
    }

    // Validate cohort date range
    if params.cohort_start_date > params.cohort_end_date {
        return actix_web::HttpResponse::BadRequest().json(ErrorResponse {
            error: std::string::String::from("Cohort start date cannot be after end date."),
        });
    }

    // Convert cohort dates to datetime
    let cohort_start_datetime = params.cohort_start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let cohort_end_datetime = params.cohort_end_date
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc();

    // Calculate event date range (default to all time if not specified)
    let (event_start_datetime, event_end_datetime) = if params.event_start_date.is_some() || params.event_end_date.is_some() || params.time_period.is_some() {
        calculate_datetime_range(
            params.event_start_date,
            params.event_end_date,
            params.time_period,
            chrono::Utc::now(),
        )
    } else {
        // Default to all time for events
        (
            chrono::DateTime::from_timestamp(0, 0).unwrap_or_else(|| chrono::Utc::now()),
            chrono::Utc::now()
        )
    };

    let result = query_cohort_registration_comprehensive(
        pool.get_ref(),
        cohort_start_datetime,
        cohort_end_datetime,
        event_start_datetime,
        event_end_datetime,
    ).await;

    match result {
        Ok(cohort_stats) => {
            if cohort_stats.is_empty() {
                let response = CohortRegistrationAnalysisResponse {
                    title: std::string::String::from("User Registration Cohort Analysis"),
                    cohort_description: format!(
                        "Users registered between {} and {}",
                        params.cohort_start_date.format("%Y-%m-%d"),
                        params.cohort_end_date.format("%Y-%m-%d")
                    ),
                    event_description: format!(
                        "Events from {} to {}",
                        event_start_datetime.format("%Y-%m-%d"),
                        event_end_datetime.format("%Y-%m-%d")
                    ),
                    dimensions: std::collections::HashMap::new(),
                };
                return actix_web::HttpResponse::Ok().json(response);
            }

            // Group stats by dimension
            let mut dimensions: std::collections::HashMap<String, std::vec::Vec<CohortRegistrationStat>> = std::collections::HashMap::new();
            for stat in cohort_stats {
                dimensions.entry(stat.dimension_name.clone())
                    .or_insert_with(std::vec::Vec::new)
                    .push(stat);
            }

            // Convert to response format with percentages
            let mut response_dimensions: std::collections::HashMap<String, std::vec::Vec<CohortDimensionStat>> = std::collections::HashMap::new();
            
            for (dimension_name, stats) in dimensions {
                let total_events_for_dimension: i64 = stats.iter().map(|s| s.total_events).sum();
                
                let dimension_stats: std::vec::Vec<CohortDimensionStat> = stats
                    .into_iter()
                    .map(|stat| {
                        let percentage = if total_events_for_dimension > 0 {
                            (stat.total_events as f64 / total_events_for_dimension as f64) * 100.0
                        } else {
                            0.0
                        };
                        
                        CohortDimensionStat {
                            value: stat.dimension_value,
                            total_events: stat.total_events,
                            unique_users: stat.unique_users,
                            percentage,
                        }
                    })
                    .collect();
                
                response_dimensions.insert(dimension_name, dimension_stats);
            }

            let response = CohortRegistrationAnalysisResponse {
                title: std::string::String::from("User Registration Cohort Analysis"),
                cohort_description: format!(
                    "Users registered between {} and {}",
                    params.cohort_start_date.format("%Y-%m-%d"),
                    params.cohort_end_date.format("%Y-%m-%d")
                ),
                event_description: format!(
                    "Events from {} to {}",
                    event_start_datetime.format("%Y-%m-%d"),
                    event_end_datetime.format("%Y-%m-%d")
                ),
                dimensions: response_dimensions,
            };
            
            actix_web::HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Failed to fetch cohort registration analysis: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: std::string::String::from("Failed to retrieve cohort registration analysis."),
            })
        }
    }
}
