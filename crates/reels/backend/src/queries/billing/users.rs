#![allow(clippy::disallowed_methods)]
//! User-related billing queries for retrieving and updating user billing information.
//!
//! This module contains database queries for user billing operations including
//! Stripe customer ID retrieval, subscription status updates, and trial management.
//! Adheres to FQN and no-`use` statements guidelines.

use sqlx::PgPool;
use uuid::Uuid;

/// Get user's Stripe customer ID
pub async fn get_user_stripe_customer_id(
    pool: &PgPool,
    user_id: Uuid,
) -> std::result::Result<Option<String>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT stripe_customer_id FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.stripe_customer_id)
}

/// Update user subscription status and end trial
pub async fn activate_user_subscription(
    pool: &PgPool,
    user_id: Uuid,
    stripe_customer_id: &str,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE users 
        SET subscription_status = 'active', 
            stripe_customer_id = $1, 
            trial_ended_at = NOW(),
            updated_at = NOW()
        WHERE id = $2
        "#,
        stripe_customer_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get user email by user ID
pub async fn get_user_email(
    pool: &PgPool,
    user_id: Uuid,
) -> std::result::Result<String, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT email FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.email)
}
