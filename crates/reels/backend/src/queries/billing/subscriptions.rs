#![allow(clippy::disallowed_methods)]
//! Fetches user subscription information from the users table.
//!
//! This function executes SQL queries against the database to retrieve subscription
//! status, trial information, and billing details for users.
//! Adheres to FQN and no-`use` statements guidelines.

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Get user subscription status and trial information
pub async fn get_user_subscription_status(
    pool: &PgPool,
    user_id: Uuid,
) -> std::result::Result<Option<crate::db::users::User>, sqlx::Error> {
    sqlx::query_as::<_, crate::db::users::User>(
        r#"
        SELECT id, email, password_hash, stripe_customer_id, email_verified, is_admin, 
               status, feature_flags, created_at, updated_at, verification_token, 
               token_expiry, trial_started_at, trial_ended_at, subscription_status, token_version
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Update user subscription status
pub async fn update_user_subscription_status(
    pool: &PgPool,
    user_id: Uuid,
    subscription_status: &str,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE users 
        SET subscription_status = $1, updated_at = $2
        WHERE id = $3
        "#,
        subscription_status,
        Utc::now(),
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Update user trial information
pub async fn update_user_trial_info(
    pool: &PgPool,
    user_id: Uuid,
    trial_started_at: Option<DateTime<Utc>>,
    trial_ended_at: Option<DateTime<Utc>>,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE users 
        SET trial_started_at = $1, trial_ended_at = $2, updated_at = $3
        WHERE id = $4
        "#,
        trial_started_at,
        trial_ended_at,
        Utc::now(),
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get users with specific subscription status
pub async fn get_users_by_subscription_status(
    pool: &PgPool,
    subscription_status: &str,
) -> std::result::Result<Vec<crate::db::users::User>, sqlx::Error> {
    sqlx::query_as::<_, crate::db::users::User>(
        r#"
        SELECT id, email, password_hash, stripe_customer_id, email_verified, is_admin, 
               status, feature_flags, created_at, updated_at, verification_token, 
               token_expiry, trial_started_at, trial_ended_at, subscription_status, token_version
        FROM users
        WHERE subscription_status = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(subscription_status)
    .fetch_all(pool)
    .await
}
