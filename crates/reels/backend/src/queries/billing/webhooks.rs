#![allow(clippy::disallowed_methods)]
//! Fetches webhook event information from the webhook_events table.
//!
//! This function executes SQL queries against the database to retrieve webhook
//! event records, idempotency checks, and webhook analytics.
//! Adheres to FQN and no-`use` statements guidelines.

use sqlx::PgPool;
use chrono::{DateTime, Utc};

/// Get webhook events by event type
pub async fn get_webhook_events_by_type(
    pool: &PgPool,
    event_type: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> std::result::Result<Vec<crate::db::billing::WebhookEvent>, sqlx::Error> {
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);
    
    sqlx::query_as::<_, crate::db::billing::WebhookEvent>(
        r#"
        SELECT id, event_id, event_type, processed_at, created_at, checkout_session_id, promo_code
        FROM webhook_events
        WHERE event_type = $1
        ORDER BY processed_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(event_type)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Get webhook events for a specific checkout session
pub async fn get_webhook_events_by_checkout_session(
    pool: &PgPool,
    checkout_session_id: &str,
) -> std::result::Result<Vec<crate::db::billing::WebhookEvent>, sqlx::Error> {
    sqlx::query_as::<_, crate::db::billing::WebhookEvent>(
        r#"
        SELECT id, event_id, event_type, processed_at, created_at, checkout_session_id, promo_code
        FROM webhook_events
        WHERE checkout_session_id = $1
        ORDER BY processed_at DESC
        "#,
    )
    .bind(checkout_session_id)
    .fetch_all(pool)
    .await
}

/// Get webhook events processed within a date range
pub async fn get_webhook_events_by_date_range(
    pool: &PgPool,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    limit: Option<i64>,
) -> std::result::Result<Vec<crate::db::billing::WebhookEvent>, sqlx::Error> {
    let limit = limit.unwrap_or(1000);
    
    sqlx::query_as::<_, crate::db::billing::WebhookEvent>(
        r#"
        SELECT id, event_id, event_type, processed_at, created_at, checkout_session_id, promo_code
        FROM webhook_events
        WHERE processed_at BETWEEN $1 AND $2
        ORDER BY processed_at DESC
        LIMIT $3
        "#,
    )
    .bind(start_date)
    .bind(end_date)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Get webhook event analytics
pub async fn get_webhook_event_analytics(
    pool: &PgPool,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> std::result::Result<WebhookEventAnalytics, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_events,
            COUNT(DISTINCT event_type) as unique_event_types,
            COUNT(DISTINCT checkout_session_id) as unique_checkout_sessions
        FROM webhook_events
        WHERE processed_at BETWEEN $1 AND $2
        "#,
        start_date,
        end_date
    )
    .fetch_one(pool)
    .await?;

    Ok(WebhookEventAnalytics {
        total_events: result.total_events.unwrap_or(0) as i64,
        unique_event_types: result.unique_event_types.unwrap_or(0) as i64,
        unique_checkout_sessions: result.unique_checkout_sessions.unwrap_or(0) as i64,
    })
}

/// Get webhook events by promo code
pub async fn get_webhook_events_by_promo_code(
    pool: &PgPool,
    promo_code: &str,
    limit: Option<i64>,
) -> std::result::Result<Vec<crate::db::billing::WebhookEvent>, sqlx::Error> {
    let limit = limit.unwrap_or(100);
    
    sqlx::query_as::<_, crate::db::billing::WebhookEvent>(
        r#"
        SELECT id, event_id, event_type, processed_at, created_at, checkout_session_id, promo_code
        FROM webhook_events
        WHERE promo_code = $1
        ORDER BY processed_at DESC
        LIMIT $2
        "#,
    )
    .bind(promo_code)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Webhook event analytics data
#[derive(Debug, Clone)]
pub struct WebhookEventAnalytics {
    pub total_events: i64,
    pub unique_event_types: i64,
    pub unique_checkout_sessions: i64,
}
