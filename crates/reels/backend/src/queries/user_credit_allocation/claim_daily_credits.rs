#![allow(clippy::disallowed_methods)]
//! Claim daily credits for a user (for free plan users).
//!
//! This function checks if the user can claim daily credits and updates the timestamp.

use chrono::{ Utc};
use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::db::user_credit_allocation::{UserCreditAllocation};
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::schemas::user_credit_allocation_schemas::DbUserCreditAllocation;
use crate::queries::credit_transactions::create_credit_transaction;
use crate::schemas::credit_transactions_schemas::CreateCreditTransactionParams;

/// Claim daily credits for a user (for free plan users)
/// This function checks if the user can claim daily credits and updates the timestamp
#[instrument(skip(pool))]
pub async fn claim_daily_credits(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<UserCreditAllocation, Error> {
    // Use a transaction to ensure atomicity
    let mut transaction = pool.begin().await?;

    // Get the current credit allocation
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
    .await?
    .ok_or_else(|| Error::RowNotFound)?;

    // Check if user is not on non-free plan
    if current_allocation.plan_type != StripePlanType::Free.as_str() {
        return Err(Error::Protocol("Daily credits can only be claimed by free plan users".into()));
    }

    // Check if user has already claimed daily credits today
    let now = Utc::now();
    if let Some(last_claimed) = current_allocation.last_daily_credit_claimed_at {
        let last_claimed_date = last_claimed.date_naive();
        let today = now.date_naive();
        
        if last_claimed_date == today {
            return Err(Error::Protocol("Daily credits already claimed today".into()));
        }
    }

    // Calculate new credits remaining (add daily credits)
    let new_credits_remaining = &current_allocation.credits_remaining + BigDecimal::from(current_allocation.daily_credits);
    let final_credits_remaining = if new_credits_remaining > BigDecimal::from(current_allocation.credit_limit) {
        BigDecimal::from(current_allocation.credit_limit)
    } else {
        new_credits_remaining
    };

    // Calculate actual credits added (considering credit limit)
    let credits_added = &final_credits_remaining - &current_allocation.credits_remaining;

    // Update the credits and timestamp
    let updated_allocation = sqlx::query_as!(
        DbUserCreditAllocation,
        r#"
        UPDATE user_credit_allocation 
        SET credits_remaining = $1, last_daily_credit_claimed_at = $2, updated_at = CURRENT_TIMESTAMP
        WHERE user_id = $3
        RETURNING id, user_id, plan_type, daily_credits, plan_credits, credits_remaining,
                  credit_limit, last_daily_credit_claimed_at, created_at, updated_at
        "#,
        final_credits_remaining,
        now,
        user_id
    )
    .fetch_one(&mut *transaction)
    .await?;

    // Commit the transaction
    transaction.commit().await?;

    // Create transaction record for audit purposes (only if credits were actually added)
    if credits_added > BigDecimal::from(0) {
        let _transaction_result = create_credit_transaction(
            pool,
            CreateCreditTransactionParams {
                user_id,
                organization_id: None, // Daily credits are user-level, not organization-level
                credits_changed: credits_added.clone(), // Positive value indicates credit addition
                previous_balance: current_allocation.credits_remaining.clone(),
                new_balance: final_credits_remaining.clone(),
                action_source: "api".to_string(),
                action_type: "claim_daily_credits".to_string(),
                entity_id: None, // Not specified for daily credit claims
            },
        ).await?;
    }

    // Return the updated allocation
    Ok(updated_allocation.into_user_credit_allocation())
}
