//! Handler for providing user-specific or global usage statistics.
//!
//! Defines the GET `/api/dashboard/usage-statistics` endpoint.
//! This endpoint returns counts of various entities (styles, assets, documents,
//! creatives, custom formats) created by users, filterable by time frame and user.
//! Supports pagination and sorting.

//! Revision History
//! - 2025-05-19T17:41:22Z @AI: Implemented core logic for handler, date calculations, SQL queries, and tests.

// No `use` statements, use fully qualified paths.

use crate::queries::dashboard::query_usage_statistics::{
    query_usage_statistics_items, query_usage_statistics_total_count, UsageStatisticsItem,
};
use crate::routes::error_response::ErrorResponse;

/// Defines the available time frames for filtering usage statistics.
#[derive(serde::Deserialize, Debug, utoipa::ToSchema, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TimeFrame {
    Today,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    AllTime,
}

/// Parameters for requesting usage statistics.
#[derive(serde::Deserialize, Debug, utoipa::ToSchema, utoipa::IntoParams)]
pub struct UsageStatisticsParams {
    /// Optional email to filter statistics for a specific user.
    /// If not provided, statistics may be aggregated globally or for the authenticated user,
    /// depending on authorization and implementation.
    #[schema(example = "user@example.com", value_type=Option<String>, nullable = true)]
    pub email: Option<std::string::String>,
    /// Optional organization ID to filter statistics for users associated with that organization.
    /// If not provided, no organization filtering is applied.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", value_type=Option<String>, nullable = true)]
    pub organization_id: Option<uuid::Uuid>,
    /// Time frame for the statistics. Defaults to AllTime if not provided.
    pub time_frame: Option<TimeFrame>,
    /// Optional subscription status filter as comma-separated string.
    /// If not provided, no subscription status filtering is applied.
    /// Example: "active,trial" or "expired"
    #[schema(example = "active,trial", value_type=Option<String>, nullable = true)]
    pub subscription_status: Option<std::string::String>,
    /// Page number for pagination (default 1).
    #[schema(example = 1, default = 1)]
    pub page: Option<i64>,
    /// Number of items per page (default 10).
    #[schema(example = 10, default = 10)]
    pub limit: Option<i64>,
    /// Field to sort by. Supported fields: `user_email`, `styles_count`, `assets_count`,
    /// `documents_count`, `creatives_count`, `custom_formats_count`.
    /// Defaults to `user_email` if not specified.
    #[schema(example = "styles_count", default = "user_email")]
    pub sort_by: Option<std::string::String>,
    /// Sort order (`asc` or `desc`). Defaults to `asc` or `desc` based on `sort_by`.
    #[schema(example = "desc", default = "asc")]
    pub sort_order: Option<std::string::String>,
}

/// Response containing a list of usage statistics items and total count for pagination.
#[derive(serde::Serialize, Debug, utoipa::ToSchema)]
pub struct ListUsageStatisticsResponse {
    pub items: std::vec::Vec<UsageStatisticsItem>,
    pub total_count: i64,
}

// Helper function to calculate date ranges
fn calculate_date_range(
    time_frame: TimeFrame,
    now: chrono::DateTime<chrono::Utc>,
) -> (
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
) {
    use chrono::Datelike; // Allowed for methods on chrono types

    let today_start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(chrono::Utc)
        .unwrap();
    let today_end = today_start + chrono::Duration::days(1);

    match time_frame {
        TimeFrame::Today => (Some(today_start), Some(today_end)),
        TimeFrame::ThisWeek => {
            let days_from_monday = now.weekday().num_days_from_monday();
            let week_start = today_start - chrono::Duration::days(days_from_monday as i64);
            let week_end = week_start + chrono::Duration::weeks(1);
            (Some(week_start), Some(week_end))
        }
        TimeFrame::LastWeek => {
            let days_from_monday = now.weekday().num_days_from_monday();
            let current_week_start = today_start - chrono::Duration::days(days_from_monday as i64);
            let last_week_start = current_week_start - chrono::Duration::weeks(1);
            let last_week_end = current_week_start;
            (Some(last_week_start), Some(last_week_end))
        }
        TimeFrame::ThisMonth => {
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
            (Some(month_start), Some(next_month_start))
        }
        TimeFrame::LastMonth => {
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
            (Some(last_month_start), Some(last_month_end))
        }
        TimeFrame::AllTime => (None, None),
    }
}

#[utoipa::path(
    get,
   path = "/api/dashboard/usage-statistics",
   tag = "Dashboard",
   params( // Note: user_id changed to email
        ("email" = Option<String>, Query, description = "Optional email to filter statistics for a specific user. If not provided, statistics may be aggregated globally or for the authenticated user, depending on authorization and implementation.", example = "user@example.com", nullable = true),
        ("organization_id" = Option<String>, Query, description = "Optional organization ID to filter statistics for users associated with that organization. If not provided, no organization filtering is applied.", example = "550e8400-e29b-41d4-a716-446655440000", nullable = true),
        ("time_frame" = Option<TimeFrame>, Query, description = "Time frame for the statistics. Defaults to AllTime if not provided."),
        ("subscription_status" = Option<String>, Query, description = "Optional subscription status filter as comma-separated string. If not provided, no subscription status filtering is applied.", example = "active,trial", nullable = true),
        ("page" = Option<i64>, Query, description = "Page number for pagination (default 1).", example = 1),
        ("limit" = Option<i64>, Query, description = "Number of items per page (default 10).", example = 10),
        ("sort_by" = Option<String>, Query, description = "Field to sort by. Supported fields: `user_email`, `styles_count`, `assets_count`, `documents_count`, `creatives_count`, `custom_formats_count`. Defaults to `user_email` if not specified.", example = "styles_count"),
        ("sort_order" = Option<String>, Query, description = "Sort order (`asc` or `desc`). Defaults to `asc` or `desc` based on `sort_by`.", example = "desc")
   ),
   responses( // Note: Using non-fully-qualified names for utoipa schema as per zenide.md
        (status = 200, description = "Usage statistics retrieved successfully", body = ListUsageStatisticsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[actix_web::get("/usage-statistics")] // Ensure this matches the path in configure_dashboard_routes
pub async fn get_usage_statistics(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>, // Used for auth and role checks
    params: actix_web::web::Query<UsageStatisticsParams>,
) -> impl actix_web::Responder {
    // Add admin check at the beginning of the function.
    if !claims.is_admin {
        return actix_web::HttpResponse::Unauthorized().json(
            crate::routes::error_response::ErrorResponse {
                // Fully qualified path
                error: std::string::String::from(
                    "Administrator access required to view usage statistics.",
                ),
            },
        );
    }

    let page = params.page.unwrap_or(1).max(1);
    let limit_val_param = params.limit.unwrap_or(10).max(1);
    let offset_val_param = (page - 1) * limit_val_param;

    let sort_by_str = params
        .sort_by
        .clone()
        .unwrap_or_else(|| "user_email".to_string());
    let sort_order_str = params
        .sort_order
        .clone()
        .unwrap_or_else(|| "asc".to_string());

    let valid_sort_fields = [
        "user_email",
        "user_created_at",
        "styles_count",
        "assets_count",
        "documents_count",
        "creatives_count",
        "custom_formats_count",
    ];
    if !valid_sort_fields.contains(&sort_by_str.as_str()) {
        return actix_web::HttpResponse::BadRequest().json(
            crate::routes::error_response::ErrorResponse {
                // Fully qualified path
                error: std::string::String::from("Invalid sort_by field."),
            },
        );
    }
    if sort_order_str != "asc" && sort_order_str != "desc" {
        return actix_web::HttpResponse::BadRequest().json(
            crate::routes::error_response::ErrorResponse {
                // Fully qualified path
                error: std::string::String::from(
                    "Invalid sort_order value. Must be 'asc' or 'desc'.",
                ),
            },
        );
    }

    let (start_date, end_date) = calculate_date_range(
        params.time_frame.unwrap_or(TimeFrame::AllTime),
        chrono::Utc::now(),
    );

    // This will store the actual value to bind if an email filter is applied by an admin.
    let mut email_filter_value: Option<std::string::String> = None;

    // Since only admins reach here (due to the check above), simplify the user filtering logic.
    // Admins can filter by email if provided.
    if let Some(ref target_email) = params.email {
        email_filter_value = Some(target_email.clone());
    }

    let items_result = query_usage_statistics_items(
        pool.get_ref(),
        start_date,
        end_date,
        email_filter_value.clone(),
        params.organization_id,
        params.subscription_status.clone(),
        &sort_by_str,
        &sort_order_str,
        limit_val_param,
        offset_val_param,
    )
    .await;

    let items = match items_result {
        Ok(items) => items,
        Err(e) => {
            log::error!("Failed to fetch usage statistics items: {e:?}");
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to retrieve usage statistics."),
                },
            );
        }
    };

    let total_count_result = query_usage_statistics_total_count(
        pool.get_ref(),
        email_filter_value,
        params.organization_id,
        params.subscription_status.clone(),
    )
    .await;
    let total_count = match total_count_result {
        Ok(count) => count,
        Err(e) => {
            log::error!("Failed to fetch usage statistics total count: {e:?}");
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to retrieve usage statistics count."),
                },
            );
        }
    };

    actix_web::HttpResponse::Ok().json(ListUsageStatisticsResponse { items, total_count })
}

#[cfg(test)]
mod tests {
    // Note: Full DB integration tests require a test database setup or mocking PgPool.
    // These tests focus on the calculate_date_range helper.
    // Add more tests for other TimeFrame variants (ThisWeek, LastWeek, ThisMonth, LastMonth, AllTime)
    // Example for AllTime:
    #[test]
    fn test_calculate_date_range_all_time() {
        let now = chrono::Utc::now();
        let (start, end) = super::calculate_date_range(super::TimeFrame::AllTime, now);
        assert_eq!(start, None);
        assert_eq!(end, None);
    }

    // TODO: Add tests for parameter validation in get_usage_statistics (sort_by, sort_order).
    // TODO: Add tests for SQL query construction (mocked or with a test DB).
}
