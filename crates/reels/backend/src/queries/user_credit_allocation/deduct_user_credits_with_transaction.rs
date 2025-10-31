#![allow(clippy::disallowed_methods)]
//! Deduct credits from user's remaining credits and log the transaction.
//!
//! This function ensures credits_remaining never goes below 0 and creates
//! a transaction record for audit purposes. For old users (created before 2024-12-01 
//! with no subscription/credit records), skips all deductions and returns immediately.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use chrono::Utc;
use bigdecimal::BigDecimal;

use crate::db::user_credit_allocation::UserCreditAllocation;
use crate::queries::credit_transactions::create_credit_transaction;
use crate::schemas::credit_transactions_schemas::CreateCreditTransactionParams;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;

/// Parameters for credit deduction with transaction logging
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CreditChangesParams {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub user_id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub organization_id: Option<Uuid>,
    
    #[schema(example = "1.0", value_type = String)]
    pub credits_to_change: BigDecimal,
    
    #[schema(example = "api")]
    pub action_source: String,
    
    #[schema(example = "retouch_images")]
    pub action_type: String,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub entity_id: Option<Uuid>,
}

/// Deduct credits from user's or organization's remaining credits and log the transaction
/// This function checks if organization_id is provided and routes to the appropriate deduction
#[instrument(skip(pool))]
pub async fn deduct_user_credits_with_transaction(
    pool: &PgPool,
    params: CreditChangesParams,
) -> Result<UserCreditAllocation, Error> {
    // CRITICAL: Check if user has unlimited access FIRST (either via new grants table OR old user logic)
    // This must happen before ANY credit deduction logic (user or organization)
    
    // Check new unlimited access grants table
    let has_unlimited_grant = crate::queries::unlimited_access::check_user_unlimited::check_user_unlimited(pool, params.user_id).await?;
    
    // Check old user exempt logic (for backwards compatibility with users created before unlimited_access_grants table)
    let is_old_user_exempt = crate::queries::user_credit_allocation::is_old_user_exempt_from_credit_checks::is_old_user_exempt_from_credit_checks(pool, params.user_id).await?;
    
    if has_unlimited_grant || is_old_user_exempt {
        log::info!(
            "User {} has unlimited access (grant={}, old_user_exempt={}), skipping all credit deduction (organization_id: {:?}, action: {}:{})",
            params.user_id,
            has_unlimited_grant,
            is_old_user_exempt,
            params.organization_id,
            params.action_source,
            params.action_type
        );
        
        // Return a dummy allocation for unlimited users - they have unlimited access
        // No deduction, no transaction logging - completely free operation
        return Ok(UserCreditAllocation {
            id: Uuid::new_v4(), // Synthetic ID
            user_id: params.user_id,
            plan_type: StripePlanType::Free,
            daily_credits: 0,
            plan_credits: 0,
            credits_remaining: BigDecimal::from(0), // Unlimited users show 0 credits (not tracked)
            credit_limit: 0,
            last_daily_credit_claimed_at: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        });
    }
    
    // If organization_id is provided, deduct from organization credits instead
    if let Some(org_id) = params.organization_id {
        log::info!(
            "Deducting {} credits from organization {} (requested by user {})",
            params.credits_to_change,
            org_id,
            params.user_id
        );
        
        // Deduct from organization credits
        let (current_org_allocation, updated_org_allocation) = 
            crate::queries::organization_credit_allocation::deduct_organization_credits::deduct_organization_credits(
                pool,
                org_id,
                params.credits_to_change.clone(),
            ).await?;
        
        // Create transaction record for organization
        let _transaction_result = create_credit_transaction(
            pool,
            CreateCreditTransactionParams {
                user_id: params.user_id,
                organization_id: Some(org_id),
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
            org_id,
            params.user_id,
            params.credits_to_change,
            current_org_allocation.credits_remaining,
            updated_org_allocation.credits_remaining,
            params.action_source.clone(),
            params.action_type.clone()
        );
        
        // For compatibility, return a user allocation (even though we deducted from org)
        // This is a workaround since the function signature returns UserCreditAllocation
        // In a future refactor, this function should return an enum or different type
        let user_allocation = crate::queries::user_credit_allocation::get_user_credit_allocation_by_user_id::get_user_credit_allocation_by_user_id(
            pool,
            params.user_id,
        ).await?;
        
        match user_allocation {
            Some(allocation) => return Ok(allocation),
            None => {
                // User has no personal credits, but that's OK when using org credits
                // Return a synthetic allocation representing org credits
                log::info!(
                    "User {} has no personal credit allocation (org-only user), returning synthetic allocation",
                    params.user_id
                );
                return Ok(UserCreditAllocation {
                    id: Uuid::new_v4(), // Synthetic ID
                    user_id: params.user_id,
                    credits_remaining: updated_org_allocation.credits_remaining.clone(),
                    plan_type: crate::schemas::user_credit_allocation_schemas::StripePlanType::Free,
                    credit_limit: updated_org_allocation.credits_remaining.clone().to_string().parse::<i32>().unwrap_or(0),
                    daily_credits: 0,
                    plan_credits: updated_org_allocation.credits_remaining.clone().to_string().parse::<i32>().unwrap_or(0),
                    last_daily_credit_claimed_at: None,
                    created_at: Some(chrono::Utc::now()),
                    updated_at: Some(chrono::Utc::now()),
                });
            }
        }
    }
    
    // Otherwise, deduct from user's personal credits
    log::info!(
        "Deducting {} credits from user {}'s personal account",
        params.credits_to_change,
        params.user_id
    );
    
    // Deduct credits and get both current and updated allocations
    let credits_to_change_clone = params.credits_to_change.clone();
    let (current_allocation, updated_allocation) = crate::queries::user_credit_allocation::deduct_user_credits::deduct_user_credits(
        pool,
        params.user_id,
        params.credits_to_change,
    ).await?;

    // Create transaction record
    // Note: credits_changed should be negative for deductions
    let _transaction_result = create_credit_transaction(
        pool,
        CreateCreditTransactionParams {
            user_id: params.user_id,
            organization_id: params.organization_id,
            credits_changed: -credits_to_change_clone.clone(),
            previous_balance: current_allocation.credits_remaining.clone(),
            new_balance: updated_allocation.credits_remaining.clone(),
            action_source: params.action_source.clone(),
            action_type: params.action_type.clone(),
            entity_id: params.entity_id,
        },
    ).await?;

    log::info!(
        "User credit transaction logged: user_id={}, credits_deducted={}, previous_balance={}, new_balance={}, action={}:{}",
        params.user_id,
        credits_to_change_clone,
        current_allocation.credits_remaining,
        updated_allocation.credits_remaining,
        params.action_source.clone(),
        params.action_type.clone()
    );

    // Return the updated allocation
    Ok(updated_allocation)
}
