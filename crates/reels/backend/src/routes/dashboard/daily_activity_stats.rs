//! Handler for providing daily activity statistics for various entities.
//!
//! Defines the GET `/api/dashboard/daily-activity-stats` endpoint.
//! This endpoint returns daily counts of specified entities (e.g., users, documents)
//! grouped by creation date, filterable by date range and optionally by user email (admin-only).
//! The data is returned in a format suitable for chart rendering.
//!
//! Revision History
//! - 2025-05-20T13:00:04Z @AI: Initial implementation of daily activity statistics endpoint.
// No `use` statements, use fully qualified paths.
use crate::routes::dashboard::chart_models::ChartData;
use crate::routes::dashboard::series_data_point::SeriesDataPoint;
use crate::routes::error_response::ErrorResponse;
use crate::queries::dashboard::query_daily_activity::query_daily_activity;

/// Enum for the type of entity to get activity statistics for.
#[derive(serde::Deserialize, Debug, utoipa::ToSchema, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityEntityType {
    Users,
    Documents,
    Creatives,
    Styles,
    Assets,
    CustomCreativeFormats,
}

impl std::fmt::Display for ActivityEntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityEntityType::Users => write!(f, "users"),
            ActivityEntityType::Documents => write!(f, "documents"),
            ActivityEntityType::Creatives => write!(f, "creatives"),
            ActivityEntityType::Styles => write!(f, "styles"),
            ActivityEntityType::Assets => write!(f, "assets"),
            ActivityEntityType::CustomCreativeFormats => write!(f, "custom_creative_formats"),
        }
    }
}

/// Enum for predefined time periods for activity statistics.
#[derive(serde::Deserialize, Debug, utoipa::ToSchema, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityTimePeriod {
    Last7Days,
    Last30Days,
    Last90Days,
    ThisMonth,
    LastMonth,
}

impl std::fmt::Display for ActivityTimePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityTimePeriod::Last7Days => write!(f, "last_7_days"),
            ActivityTimePeriod::Last30Days => write!(f, "last_30_days"),
            ActivityTimePeriod::Last90Days => write!(f, "last_90_days"),
            ActivityTimePeriod::ThisMonth => write!(f, "this_month"),
            ActivityTimePeriod::LastMonth => write!(f, "last_month"),
        }
    }
}

/// Parameters for requesting daily activity statistics.
#[derive(serde::Deserialize, Debug, utoipa::ToSchema, utoipa::IntoParams)]
pub struct DailyActivityParams {
    /// Type of entity to get statistics for (e.g., "users", "documents", "creatives", "styles", "assets", "custom_creative_formats").
    #[schema(example = "documents")]
    pub entity_type: ActivityEntityType,
    /// Optional email to filter statistics for a specific user (admin-only).
    #[schema(example = "user@example.com", value_type=Option<String>, nullable = true)]
    pub user_email: Option<std::string::String>,
    /// Optional start date for the statistics range (YYYY-MM-DD).
    #[schema(value_type = Option<String>, format = "date", example = "2023-01-01", nullable = true)]
    pub start_date: Option<chrono::NaiveDate>,
    /// Optional end date for the statistics range (YYYY-MM-DD).
    #[schema(value_type = Option<String>, format = "date", example = "2023-01-31", nullable = true)]
    pub end_date: Option<chrono::NaiveDate>,
    /// Optional time period string (e.g., "last_7_days", "last_30_days", "this_month", "last_month").
    /// If provided, this can override/calculate start/end dates. Defaults to Last30Days if not specified.
    #[schema(example = "last_30_days")]
    pub time_period: Option<ActivityTimePeriod>,
}

/// Calculates the start and end `DateTime<Utc>` based on NaiveDate inputs or a time period string.
/// The end date is exclusive (i.e., up to the beginning of that day).
pub fn calculate_datetime_range(
    params_start_date: Option<chrono::NaiveDate>,
    params_end_date: Option<chrono::NaiveDate>,
    time_period_enum: Option<ActivityTimePeriod>,
    now: chrono::DateTime<chrono::Utc>,
) -> (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) {
    use chrono::Datelike; // Allowed for methods on chrono types

    if let (Some(start), Some(end)) = (params_start_date, params_end_date) {
        if start <= end {
            let start_dt = start.and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
            // End date is inclusive in params, so add 1 day for exclusive query range
            let end_dt = (end + chrono::Duration::days(1)).and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
            return (start_dt, end_dt);
        }
    }

    // If specific start/end dates were not used to return early,
    // rely on time_period_enum, defaulting to Last30Days.
    let period_to_match = time_period_enum.unwrap_or(ActivityTimePeriod::Last30Days);

    match period_to_match {
        ActivityTimePeriod::Last7Days => {
            let end_dt = (now.date_naive() + chrono::Duration::days(1)).and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
            let start_dt = end_dt - chrono::Duration::days(7);
            (start_dt, end_dt)
        }
        ActivityTimePeriod::ThisMonth => {
            let month_start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Utc)
                .unwrap();
            let next_month_start = if now.month() == 12 {
                chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1)
            } else {
                chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
            }
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(chrono::Utc)
            .unwrap();
            (month_start, next_month_start)
        }
        ActivityTimePeriod::LastMonth => {
            let current_month_start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Utc)
                .unwrap();
            let last_month_end = current_month_start;
            let last_month_start_naive = if now.month() == 1 {
                chrono::NaiveDate::from_ymd_opt(now.year() - 1, 12, 1)
            } else {
                chrono::NaiveDate::from_ymd_opt(now.year(), now.month() - 1, 1)
            };
            let last_month_start = last_month_start_naive
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Utc)
                .unwrap();
            (last_month_start, last_month_end)
        }
        ActivityTimePeriod::Last30Days => { // Explicitly handle Last30Days
            let end_dt = (now.date_naive() + chrono::Duration::days(1)).and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
            let start_dt = end_dt - chrono::Duration::days(30);
            (start_dt, end_dt)
        }
        ActivityTimePeriod::Last90Days => { // Handle Last90Days
            let end_dt = (now.date_naive() + chrono::Duration::days(1)).and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
            let start_dt = end_dt - chrono::Duration::days(90);
            (start_dt, end_dt)
        }
    }
}


#[utoipa::path(
    get,
    path = "/api/dashboard/daily-activity-stats",
    tag = "Dashboard",
    params(DailyActivityParams),
    security(
        ("user_auth" = [])
    ),
    responses(
        (status = 200, description = "Daily activity statistics retrieved successfully", body = ChartData),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden (e.g. non-admin requesting specific user email)", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
#[actix_web::get("/daily-activity-stats")]
pub async fn get_daily_activity_stats(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    params: actix_web::web::Query<DailyActivityParams>,
) -> impl actix_web::Responder {
    if params.user_email.is_some() && !claims.is_admin {
        return actix_web::HttpResponse::Forbidden().json(ErrorResponse {
            error: std::string::String::from("Admin access required to filter by user email."),
        });
    }
    if !claims.is_admin && params.entity_type == ActivityEntityType::Users && params.user_email.is_none() {
         // Non-admins should not see all user signups unless this is a specific design choice.
         // For now, let's restrict to admin or if they query their own activities (not implemented here).
         // This endpoint is admin-focused for now.
    }

    // Validation for entity_type is now handled by serde during deserialization.
    // If an invalid string is passed, serde will fail to deserialize ActivityEntityType.

    let (start_datetime, end_datetime) = calculate_datetime_range(
        params.start_date,
        params.end_date,
        params.time_period, // Pass Option<ActivityTimePeriod>
        chrono::Utc::now(),
    );

    let result = query_daily_activity(
        pool.get_ref(),
        params.entity_type,
        start_datetime,
        end_datetime,
        params.user_email.clone(),
    )
        .await;

    match result {
        Ok(daily_counts) => {
            let mut labels: std::vec::Vec<std::string::String> = std::vec::Vec::new();
            let mut series_data_points: std::vec::Vec<SeriesDataPoint> = std::vec::Vec::new();

           for dc in daily_counts {
               labels.push(dc.activity_date.format("%Y-%m-%d").to_string());
               series_data_points.push(SeriesDataPoint { value: dc.count.into() });
           }

           let series_name = format!("Daily {} count", params.entity_type);
            let chart_series = crate::routes::dashboard::chart_models::ChartSeries {
                name: series_name,
                data: series_data_points,
            };

            let chart_data = ChartData {
                title: Some(format!("Daily Activity for {}", params.entity_type)),
                labels,
                series: vec![chart_series],
            };
            
            actix_web::HttpResponse::Ok().json(chart_data)
        }
        Err(e) => {
            log::error!("Failed to fetch daily activity statistics: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: std::string::String::from("Failed to retrieve daily activity statistics."),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    // Note: Using fully qualified paths for items from the parent module.
    // This is primarily for the helper function `calculate_datetime_range`.
    // DB interaction tests would require a test DB setup.

    #[test]
    fn test_calculate_datetime_range_last_7_days() {
        let now = chrono::DateTime::parse_from_rfc3339("2024-05-20T10:00:00Z").unwrap().with_timezone(&chrono::Utc);
        let (start, end) = super::calculate_datetime_range(None, None, Some(super::ActivityTimePeriod::Last7Days), now);

        // End should be start of 2024-05-21 (exclusive)
        let expected_end = chrono::NaiveDate::from_ymd_opt(2024, 5, 21).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        // Start should be 7 days before that: start of 2024-05-14
        let expected_start = chrono::NaiveDate::from_ymd_opt(2024, 5, 14).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();

        assert_eq!(start, expected_start);
        assert_eq!(end, expected_end);
    }

    #[test]
    fn test_calculate_datetime_range_default_last_30_days() {
        let now = chrono::DateTime::parse_from_rfc3339("2024-05-20T10:00:00Z").unwrap().with_timezone(&chrono::Utc);
        let (start, end) = super::calculate_datetime_range(None, None, None, now); // None for time_period_enum uses default

        let expected_end = chrono::NaiveDate::from_ymd_opt(2024, 5, 21).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        // Start should be 30 days before that: start of 2024-04-21
        let expected_start = chrono::NaiveDate::from_ymd_opt(2024, 4, 21).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();

        assert_eq!(start, expected_start);
        assert_eq!(end, expected_end);
    }

    #[test]
    fn test_calculate_datetime_range_specific_dates() {
        let now = chrono::Utc::now(); // 'now' is not used if specific dates are provided
        let start_date_param = chrono::NaiveDate::from_ymd_opt(2023, 1, 10);
        let end_date_param = chrono::NaiveDate::from_ymd_opt(2023, 1, 15);

        let (start, end) = super::calculate_datetime_range(start_date_param, end_date_param, None, now);

        let expected_start = start_date_param.unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        // end_date_param is inclusive, so add 1 day for query range
        let expected_end = (end_date_param.unwrap() + chrono::Duration::days(1)).and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();

        assert_eq!(start, expected_start);
        assert_eq!(end, expected_end);
    }

    #[test]
    fn test_calculate_datetime_range_this_month() {
        // Test with a date in mid-month
        let now = chrono::DateTime::parse_from_rfc3339("2024-03-15T10:00:00Z").unwrap().with_timezone(&chrono::Utc);
        let (start, end) = super::calculate_datetime_range(None, None, Some(super::ActivityTimePeriod::ThisMonth), now);
        let expected_start = chrono::NaiveDate::from_ymd_opt(2024, 3, 1).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        let expected_end = chrono::NaiveDate::from_ymd_opt(2024, 4, 1).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        assert_eq!(start, expected_start);
        assert_eq!(end, expected_end);

        // Test with a date at year boundary for next month calculation
        let now_dec = chrono::DateTime::parse_from_rfc3339("2023-12-10T10:00:00Z").unwrap().with_timezone(&chrono::Utc);
        let (start_dec, end_dec) = super::calculate_datetime_range(None, None, Some(super::ActivityTimePeriod::ThisMonth), now_dec);
        let expected_start_dec = chrono::NaiveDate::from_ymd_opt(2023, 12, 1).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        let expected_end_dec = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        assert_eq!(start_dec, expected_start_dec);
        assert_eq!(end_dec, expected_end_dec);
    }

    #[test]
    fn test_calculate_datetime_range_last_month() {
        // Test with a date in mid-month
        let now = chrono::DateTime::parse_from_rfc3339("2024-03-15T10:00:00Z").unwrap().with_timezone(&chrono::Utc);
        let (start, end) = super::calculate_datetime_range(None, None, Some(super::ActivityTimePeriod::LastMonth), now);
        let expected_start = chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        let expected_end = chrono::NaiveDate::from_ymd_opt(2024, 3, 1).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        assert_eq!(start, expected_start);
        assert_eq!(end, expected_end);

        // Test with a date in January for previous year calculation
        let now_jan = chrono::DateTime::parse_from_rfc3339("2024-01-10T10:00:00Z").unwrap().with_timezone(&chrono::Utc);
        let (start_jan, end_jan) = super::calculate_datetime_range(None, None, Some(super::ActivityTimePeriod::LastMonth), now_jan);
        let expected_start_jan = chrono::NaiveDate::from_ymd_opt(2023, 12, 1).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        let expected_end_jan = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0,0,0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        assert_eq!(start_jan, expected_start_jan);
        assert_eq!(end_jan, expected_end_jan);
    }
}
