#![allow(clippy::disallowed_methods)]
//! Create or update user credit allocation for one-time payment with transaction logging.
//!
//! This function creates or updates a user credit allocation record for one-time payments
//! and logs the credit transaction for audit purposes.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::db::user_credit_allocation::UserCreditAllocation;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::queries::user_credit_allocation::create_or_update_user_credit_allocation_for_one_time_payment;
use crate::queries::credit_transactions::create_credit_transaction;
use crate::schemas::credit_transactions_schemas::CreateCreditTransactionParams;

/// Create or update user credit allocation for one-time payment with transaction logging
#[instrument(skip(pool))]
pub async fn create_or_update_user_credit_allocation_for_one_time_payment_with_transaction(
    pool: &PgPool,
    user_id: Uuid,
    plan_type: StripePlanType,
    daily_credits: i32,
    plan_credits: i32,
    credit_limit: i32,
    organization_id: Option<Uuid>,
) -> Result<UserCreditAllocation, Error> {
    // Get current allocation to track changes
    let current_allocation = crate::queries::user_credit_allocation::get_user_credit_allocation_by_user_id::get_user_credit_allocation_by_user_id(pool, user_id).await?;
    
    // Create or update the allocation
    let updated_allocation = create_or_update_user_credit_allocation_for_one_time_payment(
        pool,
        user_id,
        plan_type,
        daily_credits,
        plan_credits,
        credit_limit,
    ).await?;

    // Calculate credits added (if any)
    let credits_added = if let Some(ref current) = current_allocation {
        &updated_allocation.credits_remaining - &current.credits_remaining
    } else {
        updated_allocation.credits_remaining.clone()
    };

    // Log transaction if credits were added
    if credits_added > BigDecimal::from(0) {
        // Create transaction record for credit addition
        let _transaction_result = create_credit_transaction(
            pool,
            CreateCreditTransactionParams {
                user_id,
                organization_id,
                credits_changed: credits_added.clone(), // Positive value indicates credit addition
                previous_balance: current_allocation.as_ref().map(|a| a.credits_remaining.clone()).unwrap_or(BigDecimal::from(0)),
                new_balance: updated_allocation.credits_remaining.clone(),
                action_source: "stripe_webhook_event".to_string(),
                action_type: "buy_credits".to_string(),
                entity_id: Some(updated_allocation.id),
            },
        ).await?;

        log::info!(
            "One-time payment credit allocation transaction logged: user_id={}, credits_added={}, previous_balance={}, new_balance={}",
            user_id,
            credits_added,
            current_allocation.as_ref().map(|a| a.credits_remaining.clone()).unwrap_or(BigDecimal::from(0)),
            updated_allocation.credits_remaining
        );
    }

    Ok(updated_allocation)
}
