#![allow(clippy::disallowed_methods)]
//! Deduct credits from organization's remaining credits and log the transaction.
//!
//! This function ensures credits_remaining never goes below 0 and creates
//! a transaction record for audit purposes.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::db::organization_credit_allocation::OrganizationCreditAllocation;
use crate::queries::credit_transactions::create_credit_transaction;
use crate::schemas::credit_transactions_schemas::CreateCreditTransactionParams;

/// Parameters for organization credit deduction with transaction logging
#[derive(Debug, Clone)]
pub struct OrganizationCreditChangesParams {
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub credits_to_change: BigDecimal,
    pub action_source: String,
    pub action_type: String,
    pub entity_id: Option<Uuid>,
}

/// Deduct credits from organization's remaining credits and log the transaction
#[instrument(skip(pool))]
pub async fn deduct_organization_credits_with_transaction(
    pool: &PgPool,
    params: OrganizationCreditChangesParams,
) -> Result<OrganizationCreditAllocation, Error> {
    log::info!(
        "Deducting {} credits from organization {} (requested by user {})",
        params.credits_to_change,
        params.organization_id,
        params.user_id
    );
    
    // Deduct from organization credits
    let (current_org_allocation, updated_org_allocation) = 
        crate::queries::organization_credit_allocation::deduct_organization_credits::deduct_organization_credits(
            pool,
            params.organization_id,
            params.credits_to_change.clone(),
        ).await?;
    
    // Create transaction record for organization
    let _transaction_result = create_credit_transaction(
        pool,
        CreateCreditTransactionParams {
            user_id: params.user_id,
            organization_id: Some(params.organization_id),
            credits_changed: -params.credits_to_change.clone(),
            previous_balance: current_org_allocation.credits_remaining.clone(),
            new_balance: updated_org_allocation.credits_remaining.clone(),
            action_source: params.action_source.clone(),
            action_type: params.action_type.clone(),
            entity_id: params.entity_id,
        },
    ).await?;
    
    log::info!(
        "Organization credit transaction logged: org_id={}, user_id={}, credits_deducted={}, previous_balance={}, new_balance={}, action={}:{}",
        params.organization_id,
        params.user_id,
        params.credits_to_change,
        current_org_allocation.credits_remaining,
        updated_org_allocation.credits_remaining,
        params.action_source.clone(),
        params.action_type.clone()
    );

    // Return the updated organization allocation
    Ok(updated_org_allocation)
}
