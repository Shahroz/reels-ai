//! Defines queries for fetching custom events statistics from analytics_events table
//! 
//! Provides functionality to aggregate event counts and unique user counts
//! grouped by event_name within specified time periods.
//!
//! Revision History
//! - 2025-09-16T00:00:00Z @AI: Initial implementation of custom events statistics query.

/// Represents aggregated statistics for a single event type
#[derive(Debug)]
pub struct CustomEventStat {
    pub event_name: String,
    pub total_events: i64,
    pub unique_users: i64,
}

/// Fetches custom event statistics within a date range
#[tracing::instrument(
    name = "query_custom_events_stats",
    skip(pool),
    fields(
        start_date = %start_datetime,
        end_date = %end_datetime
    )
)]
pub async fn query_custom_events_stats(
    pool: &sqlx::PgPool,
    start_datetime: chrono::DateTime<chrono::Utc>,
    end_datetime: chrono::DateTime<chrono::Utc>,
) -> Result<std::vec::Vec<CustomEventStat>, sqlx::Error> {

    let rows = sqlx::query!(
        r#"
        SELECT 
            event_name,
            COUNT(*) as total_events,
            COUNT(DISTINCT user_id) as unique_users
        FROM analytics_events 
        WHERE timestamp >= $1 AND timestamp < $2
        GROUP BY event_name
        ORDER BY total_events DESC, event_name ASC
        "#,
        start_datetime,
        end_datetime
    )
    .fetch_all(pool)
    .await;

    let results = rows.map(|rows| {
        rows.into_iter()
            .map(|row| CustomEventStat {
                event_name: row.event_name,
                total_events: row.total_events.unwrap_or(0),
                unique_users: row.unique_users.unwrap_or(0),
            })
            .collect::<Vec<CustomEventStat>>()
    });

    log::debug!("Custom events query executed: {} events found", 
        results.as_ref().map(|r: &Vec<CustomEventStat>| r.len()).unwrap_or(0));

    results
}
