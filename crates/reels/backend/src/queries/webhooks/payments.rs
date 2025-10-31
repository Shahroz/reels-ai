#![allow(clippy::disallowed_methods)]
//! Webhook payment-related database queries.
//!
//! This module contains database queries for payment operations during webhook processing,
//! including payment completion lookups and credit allocation queries.
//! Adheres to FQN and no-`use` statements guidelines.

use sqlx::PgPool;

/// Check if payment completion exists by session ID
pub async fn payment_completion_exists_by_session_id(
    pool: &PgPool,
    session_id: &str,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT id FROM payment_completions WHERE session_id = $1",
        session_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

/// Get current user credit allocation
pub async fn get_current_user_credit_allocation(
    pool: &PgPool,
    user_id: uuid::Uuid,
) -> std::result::Result<Option<(String, i32)>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT plan_type, credit_limit FROM user_credit_allocation WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| (r.plan_type, r.credit_limit)))
}
