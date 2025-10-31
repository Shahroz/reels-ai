#![allow(clippy::disallowed_methods)]
//! Create or update user credit allocation with transaction logging.
//!
//! This function creates or updates a user credit allocation record and logs
//! the credit transaction for audit purposes.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::db::user_credit_allocation::UserCreditAllocation;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::queries::user_credit_allocation::{create_or_update_user_credit_allocation, CreditChangesParams};
use crate::queries::credit_transactions::create_credit_transaction;
use crate::schemas::credit_transactions_schemas::CreateCreditTransactionParams;

/// Create or update user credit allocation with transaction logging
#[instrument(skip(pool))]
pub async fn create_or_update_user_credit_allocation_with_transaction(
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
    let updated_allocation = create_or_update_user_credit_allocation(
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
        let transaction_params = CreditChangesParams {
            user_id,
            organization_id,
            credits_to_change: credits_added.clone(), // Positive value indicates credit addition
            action_source: if plan_type == StripePlanType::Free { "free_subscription".to_string() } else { "stripe_webhook_event".to_string() },
            action_type: if plan_type == StripePlanType::Free { "allocate_free_credits".to_string() } else { "buy_credits".to_string() },
            entity_id: Some(updated_allocation.id),
        };

        // Create transaction record for credit addition
        let _transaction_result = create_credit_transaction(
            pool,
            CreateCreditTransactionParams {
                user_id: transaction_params.user_id,
                organization_id: transaction_params.organization_id,
                credits_changed: credits_added.clone(), // Positive value indicates credit addition
                previous_balance: current_allocation.as_ref().map(|a| a.credits_remaining.clone()).unwrap_or(BigDecimal::from(0)),
                new_balance: updated_allocation.credits_remaining.clone(),
                action_source: transaction_params.action_source.clone(),
                action_type: transaction_params.action_type.clone(),
                entity_id: transaction_params.entity_id,
            },
        ).await?;

        log::info!(
            "Credit allocation transaction logged: user_id={}, credits_added={}, previous_balance={}, new_balance={}, action_source={}, action_type={}",
            user_id,
            credits_added,
            current_allocation.as_ref().map(|a| a.credits_remaining.clone()).unwrap_or(BigDecimal::from(0)),
            updated_allocation.credits_remaining,
            transaction_params.action_source,
            transaction_params.action_type
        );
    }

    Ok(updated_allocation)
}
