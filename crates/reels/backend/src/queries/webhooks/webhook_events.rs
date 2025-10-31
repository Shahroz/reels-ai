#![allow(clippy::disallowed_methods)]
//! Webhook event-related database queries.
//!
//! This module contains database queries for webhook event operations,
//! including promo code lookups and event tracking.
//! Adheres to FQN and no-`use` statements guidelines.

use sqlx::PgPool;

/// Get latest promo code from webhook events
pub async fn get_latest_promo_code_from_webhook_events(
    pool: &PgPool,
) -> std::result::Result<Option<String>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT promo_code FROM webhook_events WHERE checkout_session_id IS NOT NULL AND promo_code IS NOT NULL ORDER BY processed_at DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.and_then(|r| r.promo_code))
}
