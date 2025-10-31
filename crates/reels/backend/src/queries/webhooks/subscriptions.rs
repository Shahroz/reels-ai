#![allow(clippy::disallowed_methods)]
//! Webhook subscription-related database queries.
//!
//! This module contains database queries for subscription operations during webhook processing,
//! including subscription lookups, updates, and status checks.
//! Adheres to FQN and no-`use` statements guidelines.

use sqlx::PgPool;
use uuid::Uuid;

/// Get active subscription count for user
pub async fn get_active_subscription_count_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> std::result::Result<i64, sqlx::Error> {
    let count: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(id) FROM user_subscriptions WHERE user_id = $1 AND status = 'active'"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0))
}

/// Get subscription count by Stripe product ID
pub async fn get_subscription_count_by_stripe_product_id(
    pool: &PgPool,
    stripe_product_id: &str,
) -> std::result::Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT COUNT(id) as count FROM user_subscriptions WHERE stripe_product_id = $1",
        stripe_product_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.count.unwrap_or(0))
}

/// Update stripe plan type for subscriptions by product ID
pub async fn update_stripe_plan_type_by_product_id(
    pool: &PgPool,
    stripe_product_id: &str,
    plan_type: &str,
) -> std::result::Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE user_subscriptions 
        SET stripe_plan_type = $1, updated_at = NOW()
        WHERE stripe_product_id = $2
        "#,
        plan_type,
        stripe_product_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
