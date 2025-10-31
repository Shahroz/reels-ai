//! Defines the query logic for fetching Key Performance Indicator (KPI) metrics.
//!
//! This module contains the functions and data structures necessary to query
//! the database for various time-based count metrics for different entities,
//! and to calculate percentage changes between periods.
//!
//! Revision History
//! - 2025-06-18T17:52:23Z @USER: Refactored from kpi_summary.rs route handler.

/// Represents a single Key Performance Indicator metric with various time-based counts.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct KpiMetric {
    /// User-friendly name of the metric (e.g., "Users", "Documents").
    #[schema(example = "Users")]
    pub name: std::string::String,
    /// Total count of items ever for this metric.
    #[schema(example = 1250)]
    pub total_count: i64,
    /// Count of items created today.
    #[schema(example = 5)]
    pub today: i64,
    /// Count of items created yesterday.
    #[schema(example = 3)]
    pub yesterday: i64,
    /// Count of items created this week (since Monday UTC).
    #[schema(example = 25)]
    pub this_week: i64,
    /// Count of items created last week (Monday to Sunday UTC).
    #[schema(example = 22)]
    pub last_week: i64,
    /// Count of items created this month (since 1st of month UTC).
    #[schema(example = 80)]
    pub this_month: i64,
    /// Count of items created last month (full previous month).
    #[schema(example = 75)]
    pub last_month: i64,
    /// Percentage change in items created in the last 7 full days (ending yesterday)
    /// compared to the 7 days prior to that.
    #[schema(example = 10.5, value_type = Option<f32>, nullable = true)]
    pub change_last_7_days_vs_prev_7_days: Option<f32>,
    /// Percentage change in items created in the last 30 full days (ending yesterday)
    /// compared to the 30 days prior to that.
    #[schema(example = -5.2, value_type = Option<f32>, nullable = true)]
    pub change_last_30_days_vs_prev_30_days: Option<f32>,
}

/// Calculates percentage change: ((current - previous) / previous) * 100.0.
/// Handles division by zero:
/// - If previous is 0 and current is 0, change is 0.0%.
/// - If previous is 0 and current > 0, change is None (representing infinity/undefined).
fn calculate_percentage_change(current: i64, previous: i64) -> Option<f32> {
    if previous == 0 {
        if current == 0 {
            Some(0.0_f32)
        } else {
            None // Undefined change (e.g. from 0 to 10 is infinite percent increase)
        }
    } else {
        Some(((current as f32 - previous as f32) / previous as f32) * 100.0_f32)
    }
}

/// Fetches count of items from a table within a specific time period.
async fn fetch_count_in_period(
    pool: &sqlx::PgPool,
    table_name: &str,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> Result<i64, sqlx::Error> {
    let query_str = format!(
        "SELECT COUNT(*) FROM public.{table_name} WHERE created_at >= $1 AND created_at < $2" // Table names are hardcoded, so this is safe.
    );
    sqlx::query_scalar(&query_str)
        .bind(start_time)
        .bind(end_time)
        .fetch_one(pool)
        .await
}

/// Fetches total count of items from a table.
async fn fetch_total_count(pool: &sqlx::PgPool, table_name: &str) -> Result<i64, sqlx::Error> {
    let query_str = format!(
        "SELECT COUNT(*) FROM public.{table_name}" // Table names are hardcoded, so this is safe.
    );
    sqlx::query_scalar(&query_str).fetch_one(pool).await
}

/// Builds a KpiMetric for a given table by fetching all necessary counts.
#[tracing::instrument(
    name = "build_kpi_metric_query",
    skip(pool, dates),
    fields(table_name = %table_name, display_name = %display_name)
)]
pub async fn build_kpi_metric(
    pool: &sqlx::PgPool,
    table_name: &str,
    display_name: &str,
    dates: &crate::routes::dashboard::kpi_summary::AllDateRanges,
) -> Result<KpiMetric, sqlx::Error> {
    let total_count_fut = fetch_total_count(pool, table_name);
    let today_fut = fetch_count_in_period(pool, table_name, dates.today_start, dates.today_end);
    let yesterday_fut = fetch_count_in_period(pool, table_name, dates.yesterday_start, dates.yesterday_end);
    let this_week_fut = fetch_count_in_period(pool, table_name, dates.this_week_start, dates.this_week_end);
    let last_week_fut = fetch_count_in_period(pool, table_name, dates.last_week_start, dates.last_week_end);
    let this_month_fut = fetch_count_in_period(pool, table_name, dates.this_month_start, dates.this_month_end);
    let last_month_fut = fetch_count_in_period(pool, table_name, dates.last_month_start, dates.last_month_end);
    let current_7d_fut = fetch_count_in_period(pool, table_name, dates.current_7_days_start, dates.current_7_days_end);
    let previous_7d_fut = fetch_count_in_period(pool, table_name, dates.previous_7_days_start, dates.previous_7_days_end);
    let current_30d_fut = fetch_count_in_period(pool, table_name, dates.current_30_days_start, dates.current_30_days_end);
    let previous_30d_fut = fetch_count_in_period(pool, table_name, dates.previous_30_days_start, dates.previous_30_days_end);

    let (
        total_count,
        today,
        yesterday,
        this_week,
        last_week,
        this_month,
        last_month,
        current_7_days,
        previous_7_days,
        current_30_days,
        previous_30_days,
    ) = futures::try_join!(
        total_count_fut,
        today_fut,
        yesterday_fut,
        this_week_fut,
        last_week_fut,
        this_month_fut,
        last_month_fut,
        current_7d_fut,
        previous_7d_fut,
        current_30d_fut,
        previous_30d_fut
    )?;

    let change_last_7_days_vs_prev_7_days = calculate_percentage_change(current_7_days, previous_7_days);
    let change_last_30_days_vs_prev_30_days = calculate_percentage_change(current_30_days, previous_30_days);

    Ok(KpiMetric {
        name: display_name.to_string(),
        total_count,
        today,
        yesterday,
        this_week,
        last_week,
        this_month,
        last_month,
        change_last_7_days_vs_prev_7_days,
        change_last_30_days_vs_prev_30_days,
    })
}