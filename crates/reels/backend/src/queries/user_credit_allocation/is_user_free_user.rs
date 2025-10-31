#![allow(clippy::disallowed_methods)]
//! Check if a user is a free user based on their subscription status in the users table.
//!
//! Free users are those with subscription_status: 'trial', 'trialing', 'expired', 'cancelled', 'canceled', or 'paused'.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

/// Check if a user is a free user based on their subscription status in the users table
/// Free users are those with subscription_status: 'trial', 'trialing', 'expired', 'cancelled', 'canceled', or 'paused'
#[instrument(skip(pool))]
pub async fn is_user_free_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        "SELECT subscription_status FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    let subscription_status = match result {
        Some(row) => row.subscription_status.unwrap_or("trialing".to_string()),
        None => return Ok(false), // User not found
    };

    // Consider users with these statuses as free users
    let free_statuses = ["trial", "trialing", "expired", "cancelled", "canceled", "paused"];
    Ok(free_statuses.contains(&subscription_status.as_str()))
}
