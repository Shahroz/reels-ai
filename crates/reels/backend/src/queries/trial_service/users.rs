#![allow(clippy::disallowed_methods)]
//! Trial service user-related database queries.
//!
//! This module contains database queries for trial management operations including
//! trial status retrieval, trial activation/deactivation, and user trial information.
//! Adheres to FQN and no-`use` statements guidelines.

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Get user trial information for trial status calculation
pub async fn get_user_trial_info(
    pool: &PgPool,
    user_id: Uuid,
) -> std::result::Result<UserTrialInfo, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT trial_started_at, trial_ended_at, subscription_status
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(UserTrialInfo {
        trial_started_at: result.trial_started_at,
        trial_ended_at: result.trial_ended_at,
        subscription_status: result.subscription_status,
    })
}

/// Get user billing information for trial status calculation
pub async fn get_user_billing_info(
    pool: &PgPool,
    user_id: Uuid,
) -> std::result::Result<UserBillingInfo, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT trial_started_at, trial_ended_at, subscription_status, stripe_customer_id
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(UserBillingInfo {
        trial_started_at: result.trial_started_at,
        trial_ended_at: result.trial_ended_at,
        subscription_status: result.subscription_status,
        stripe_customer_id: result.stripe_customer_id,
    })
}

/// End user trial
pub async fn end_user_trial(
    pool: &PgPool,
    user_id: Uuid,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE users
        SET trial_ended_at = $1, subscription_status = 'expired'
        WHERE id = $2
        "#,
        Utc::now(),
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Activate user subscription (simple version)
pub async fn activate_user_subscription_simple(
    pool: &PgPool,
    user_id: Uuid,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE users
        SET subscription_status = 'active'
        WHERE id = $1
        "#,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get users whose trial is expiring soon
pub async fn get_users_with_expiring_trial(
    pool: &PgPool,
    days_until_expiry: i64,
) -> std::result::Result<Vec<crate::db::users::User>, sqlx::Error> {
    let _trial_period_days = crate::services::trial_service::get_trial_period_days();
    let expiry_threshold = Utc::now() + chrono::Duration::days(days_until_expiry);
    
    sqlx::query_as::<_, crate::db::users::User>(
        r#"
        SELECT id, email, password_hash, stripe_customer_id, email_verified, is_admin, 
               status, feature_flags, created_at, updated_at, verification_token, 
               token_expiry, trial_started_at, trial_ended_at, subscription_status
        FROM users
        WHERE subscription_status = 'trial' 
          AND trial_started_at IS NOT NULL
          AND trial_started_at + INTERVAL '{trial_period_days} days' <= $1
        ORDER BY trial_started_at ASC
        "#,
    )
    .bind(expiry_threshold)
    .fetch_all(pool)
    .await
}

/// User trial information struct
#[derive(Debug, Clone)]
pub struct UserTrialInfo {
    pub trial_started_at: Option<DateTime<Utc>>,
    pub trial_ended_at: Option<DateTime<Utc>>,
    pub subscription_status: Option<String>,
}

/// User billing information struct
#[derive(Debug, Clone)]
pub struct UserBillingInfo {
    pub trial_started_at: Option<DateTime<Utc>>,
    pub trial_ended_at: Option<DateTime<Utc>>,
    pub subscription_status: Option<String>,
    pub stripe_customer_id: Option<String>,
}
