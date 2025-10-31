#![allow(clippy::disallowed_methods)]
//! Check if user is an "old user" who should skip credit limits checks.
//!
//! An old user is defined as:
//! 1. Has no records in user_subscriptions table
//! 2. Has no records in user_credit_allocation table  
//! 3. Subscription status is active

use sqlx::{Error, PgPool};
use tracing::instrument;
use uuid::Uuid;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;

/// Check if user is an "old user" who should skip credit limits checks
/// An old user is defined as:
/// 1. Has no records in user_subscriptions table
/// 2. Has no records in user_credit_allocation table  
/// 3. Subscription status is active
#[instrument(skip(pool))]
pub async fn is_old_user_exempt_from_credit_checks(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<bool, Error> {
    // Check if user has an active subscription
    let result = sqlx::query!(
        "SELECT subscription_status FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    let user_subscription_status = match result {
        Some(row) => row
            .subscription_status
            .unwrap_or(SubscriptionStatus::Trial.as_str().to_string()),
        None => return Ok(false), // User not found
    };

    // Check if user has any subscription records
    let has_subscriptions = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM user_subscriptions WHERE user_id = $1)",
        user_id
    )
    .fetch_one(pool)
    .await?;

    // Check if user has any credit allocation records, if so, then return false
    let has_credit_allocation = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM user_credit_allocation WHERE user_id = $1)",
        user_id
    )
    .fetch_one(pool)
    .await?;

    // If subscription is active or trial and user has no subscription records and no credit allocation records, then they are exempt
    if user_subscription_status == SubscriptionStatus::Active.as_str()
        && !has_subscriptions.unwrap_or(false)
        && !has_credit_allocation.unwrap_or(false)
    {
        // User is an old user exempt from credit checks
        return Ok(true);
    }

    // User is not exempt from credit checks, return false
    Ok(false)
}
