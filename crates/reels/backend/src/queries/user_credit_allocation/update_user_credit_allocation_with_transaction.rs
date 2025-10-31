#![allow(clippy::disallowed_methods)]
//! Update user credit allocation with transaction logging.
//!
//! This function updates a user credit allocation record and logs
//! the credit transaction for audit purposes.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::db::user_credit_allocation::UserCreditAllocation;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::queries::user_credit_allocation::update_user_credit_allocation;
use crate::queries::credit_transactions::create_credit_transaction;
use crate::schemas::credit_transactions_schemas::CreateCreditTransactionParams;

/// Update user credit allocation with transaction logging
#[instrument(skip(pool))]
pub async fn update_user_credit_allocation_with_transaction(
    pool: &PgPool,
    user_id: Uuid,
    plan_type: StripePlanType,
    daily_credits: i32,
    plan_credits: i32,
    credits_remaining: BigDecimal,
    credit_limit: i32,
    organization_id: Option<Uuid>,
) -> Result<UserCreditAllocation, Error> {
    // Get current allocation to track changes
    let current_allocation = crate::queries::user_credit_allocation::get_user_credit_allocation_by_user_id::get_user_credit_allocation_by_user_id(pool, user_id).await?;
    
    // Update the allocation
    let updated_allocation = update_user_credit_allocation(
        pool,
        user_id,
        plan_type,
        daily_credits,
        plan_credits,
        credits_remaining,
        credit_limit,
    ).await?;

    // Calculate credits changed (if any)
    let credits_changed = if let Some(ref current) = current_allocation {
        &updated_allocation.credits_remaining - &current.credits_remaining
    } else {
        updated_allocation.credits_remaining.clone()
    };

    // Log transaction if credits were changed
    if credits_changed != BigDecimal::from(0) {
        // Create transaction record for credit change
        let _transaction_result = create_credit_transaction(
            pool,
            CreateCreditTransactionParams {
                user_id,
                organization_id,
                credits_changed: credits_changed.clone(), // Positive value indicates credit addition, negative indicates deduction
                previous_balance: current_allocation.as_ref().map(|a| a.credits_remaining.clone()).unwrap_or(BigDecimal::from(0)),
                new_balance: updated_allocation.credits_remaining.clone(),
                action_source: "stripe_webhook_event".to_string(),
                action_type: "buy_credits".to_string(),
                entity_id: Some(updated_allocation.id),
            },
        ).await?;

        log::info!(
            "Credit allocation update transaction logged: user_id={}, credits_changed={}, previous_balance={}, new_balance={}",
            user_id,
            credits_changed,
            current_allocation.as_ref().map(|a| a.credits_remaining.clone()).unwrap_or(BigDecimal::from(0)),
            updated_allocation.credits_remaining
        );
    }

    Ok(updated_allocation)
}
