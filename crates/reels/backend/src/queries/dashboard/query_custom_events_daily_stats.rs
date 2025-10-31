//! Defines queries for fetching daily statistics for specific custom events
//! 
//! Provides functionality to aggregate daily event counts and unique user counts
//! for a specific event_name within specified time periods.
//!
//! Revision History
//! - 2025-09-16T00:00:00Z @AI: Initial implementation of daily custom events statistics query.

/// Represents daily statistics for a specific event type
#[derive(Debug)]
pub struct CustomEventDailyStat {
    pub activity_date: chrono::NaiveDate,
    pub total_events: i64,
    pub unique_users: i64,
}

/// Fetches daily custom event statistics for a specific event type within a date range
#[tracing::instrument(
    name = "query_custom_events_daily_stats",
    skip(pool),
    fields(
        event_name = %event_name,
        start_date = %start_datetime,
        end_date = %end_datetime
    )
)]
pub async fn query_custom_events_daily_stats(
    pool: &sqlx::PgPool,
    event_name: &str,
    start_datetime: chrono::DateTime<chrono::Utc>,
    end_datetime: chrono::DateTime<chrono::Utc>,
) -> Result<std::vec::Vec<CustomEventDailyStat>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            DATE(timestamp) as activity_date,
            COUNT(*) as total_events,
            COUNT(DISTINCT user_id) as unique_users
        FROM analytics_events 
        WHERE event_name = $1 
            AND timestamp >= $2 
            AND timestamp < $3
        GROUP BY DATE(timestamp)
        ORDER BY activity_date ASC
        "#,
        event_name,
        start_datetime,
        end_datetime
    )
    .fetch_all(pool)
    .await;

    let result = rows.map(|rows| {
        rows.into_iter()
            .map(|row| CustomEventDailyStat {
                activity_date: row.activity_date.unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                total_events: row.total_events.unwrap_or(0),
                unique_users: row.unique_users.unwrap_or(0),
            })
            .collect::<Vec<CustomEventDailyStat>>()
    });

    match &result {
        Ok(stats) => {
            log::debug!(
                "Retrieved {} daily stats for event '{}' from {} to {}",
                stats.len(),
                event_name,
                start_datetime.date_naive(),
                end_datetime.date_naive()
            );
        }
        Err(e) => {
            log::error!(
                "Failed to retrieve daily stats for event '{}': {:?}",
                event_name,
                e
            );
        }
    }

    result
}
