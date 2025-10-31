#![allow(clippy::disallowed_methods)]
//! Deduct credits from user's remaining credits.
//!
//! This function ensures credits_remaining never goes below 0.
//! For old users (created before 2024-12-01 with no subscription/credit records), skips deduction.

use chrono::{Utc};
use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::db::user_credit_allocation::{UserCreditAllocation};
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::schemas::user_credit_allocation_schemas::DbUserCreditAllocation;
use crate::schemas::user_subscription_schemas::SubscriptionStatus;

/// Deduct credits from user's remaining credits
/// This function ensures credits_remaining never goes below 0
/// For old users (created before 2024-12-01 with no subscription/credit records), skips deduction
/// Returns both current and updated allocations for transaction logging
#[instrument(skip(pool))]
pub async fn deduct_user_credits(
    pool: &PgPool,
    user_id: Uuid,
    credits_to_deduct: BigDecimal,
) -> Result<(UserCreditAllocation, UserCreditAllocation), Error> {

    // Check if user has unlimited access grant
    if crate::queries::unlimited_access::check_user_unlimited::check_user_unlimited(pool, user_id).await? {
        log::info!("User {} has unlimited access grant, skipping credit deduction", user_id);
        // Return a dummy allocation for unlimited users (both current and updated are the same)
        let dummy_allocation = UserCreditAllocation {
            id: Uuid::new_v4(),
            user_id,
            plan_type: StripePlanType::Free,
            daily_credits: 0,
            plan_credits: 0,
            credits_remaining: BigDecimal::from(999999),
            credit_limit: 0,
            last_daily_credit_claimed_at: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        return Ok((dummy_allocation.clone(), dummy_allocation));
    }

    // Check if user has a subscription - if not, they might be a free user with credit allocation
    let statuses = [SubscriptionStatus::Trialing, SubscriptionStatus::Trial, SubscriptionStatus::Active];
    let user_subscription = crate::queries::users::get_user_by_statuses::get_user_by_statuses(pool, user_id, &statuses).await?;
    
    // If user has no subscription, check if they are a free user based on users.subscription_status
    if user_subscription.is_none() {
        let is_free_user = crate::queries::user_credit_allocation::is_user_free_user::is_user_free_user(pool, user_id).await?;
        if is_free_user {
            // This is a free user, check if they have a credit allocation
            let credit_allocation = crate::queries::user_credit_allocation::get_user_credit_allocation_by_user_id::get_user_credit_allocation_by_user_id(pool, user_id).await?;
            if credit_allocation.is_some() {
                // This is a free user with credit allocation, proceed with deduction
                log::info!("User {} is a free user with credit allocation, proceeding with credit deduction", user_id);
            } else {
                return Err(Error::RowNotFound);
            }
        } else {
            return Err(Error::Protocol("User does not have an active subscription or credit allocation".into()));
        }
    } else {
        // User has subscription, check if it's active
        let user_subscription = user_subscription.unwrap();
        if user_subscription.status != SubscriptionStatus::Trialing.as_str() && user_subscription.status != SubscriptionStatus::Trial.as_str() && user_subscription.status != SubscriptionStatus::Active.as_str() {
            return Err(Error::Protocol("User does not have an active subscription".into()));
        }
    }

    // Validate input - ensure we're not trying to deduct negative credits
    if credits_to_deduct < BigDecimal::from(0) {
        return Err(Error::Protocol("Cannot deduct negative credits".into()));
    }

    // Using a transaction to ensure atomicity and prevent race conditions
    let mut transaction = pool.begin().await?;

    // First, get the current credit allocation to check available credits
    let current_allocation = sqlx::query_as!(
        DbUserCreditAllocation,
        r#"
        SELECT id, user_id, plan_type, daily_credits, plan_credits, credits_remaining,
               credit_limit, last_daily_credit_claimed_at, created_at, updated_at
        FROM user_credit_allocation
        WHERE user_id = $1
        FOR UPDATE
        "#,
        user_id
    )
    .fetch_optional(&mut *transaction)
    .await?;

    let current_allocation = current_allocation
        .ok_or_else(|| Error::RowNotFound)?;

    // Check if user has enough credits
    if current_allocation.credits_remaining < credits_to_deduct {
        return Err(Error::Protocol(format!(
            "Insufficient credits. Available: {}, Requested: {}",
            current_allocation.credits_remaining,
            credits_to_deduct
        )));
    }

    // Calculate new credits remaining (guaranteed to be >= 0 due to above check)
    let new_credits_remaining = &current_allocation.credits_remaining - &credits_to_deduct;

    // Update the credits_remaining in the database
    let updated_allocation = sqlx::query_as!(
        DbUserCreditAllocation,
        r#"
        UPDATE user_credit_allocation 
        SET credits_remaining = $1, updated_at = CURRENT_TIMESTAMP
        WHERE user_id = $2
        RETURNING id, user_id, plan_type, daily_credits, plan_credits, credits_remaining,
                  credit_limit, last_daily_credit_claimed_at, created_at, updated_at
        "#,
        new_credits_remaining,
        user_id
    )
    .fetch_one(&mut *transaction)
    .await?;

    // Commit the transaction
    transaction.commit().await?;

    // Return both current and updated allocations
    let current_allocation = current_allocation.into_user_credit_allocation();
    let updated_allocation = updated_allocation.into_user_credit_allocation();
    Ok((current_allocation, updated_allocation))
}
