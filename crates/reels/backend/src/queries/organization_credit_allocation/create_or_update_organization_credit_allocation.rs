#![allow(clippy::disallowed_methods)]
//! Create or update organization credit allocation.
//!
//! This function creates a new credit allocation or updates existing one.
//! Used primarily for subscription renewal and credit refills.

use bigdecimal::BigDecimal;
use chrono::Utc;
use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::db::organization_credit_allocation::OrganizationCreditAllocation;
use crate::queries::organization_credit_allocation::get_organization_credit_allocation_by_org_id::get_organization_credit_allocation_by_org_id;
use crate::queries::credit_transactions::create_credit_transaction;
use crate::schemas::credit_transactions_schemas::CreateCreditTransactionParams;

/// Create or update organization credit allocation with transaction logging
/// 
/// This function adds credits to an organization's allocation and logs the transaction.
/// If no user_id is provided, the transaction is logged under the organization context only.
#[instrument(skip(pool))]
pub async fn create_or_update_organization_credit_allocation(
    pool: &PgPool,
    organization_id: Uuid,
    credits_to_add: BigDecimal,
    user_id: Option<Uuid>,
) -> Result<OrganizationCreditAllocation, Error> {
    // Get current allocation to track changes
    let current_allocation = get_organization_credit_allocation_by_org_id(pool, organization_id).await?;
    
    let result = sqlx::query_as!(
        OrganizationCreditAllocation,
        r#"
        INSERT INTO organization_credit_allocation (
            organization_id, credits_remaining, last_reset_date
        )
        VALUES ($1, $2, $3)
        ON CONFLICT (organization_id) 
        DO UPDATE SET 
            credits_remaining = organization_credit_allocation.credits_remaining + $2,
            last_reset_date = $3,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id, organization_id, credits_remaining, last_reset_date, created_at, updated_at
        "#,
        organization_id,
        credits_to_add,
        Utc::now()
    )
    .fetch_one(pool)
    .await?;

    // Log transaction if credits were added
    if credits_to_add > BigDecimal::from(0) {
        // Create transaction record for credit addition
        // Use organization_id as fallback if no user_id provided
        let user_id_for_transaction = user_id.unwrap_or(organization_id);
        
        let _transaction_result = create_credit_transaction(
            pool,
            CreateCreditTransactionParams {
                user_id: user_id_for_transaction,
                organization_id: Some(organization_id),
                credits_changed: credits_to_add.clone(), // Positive value indicates credit addition
                previous_balance: current_allocation.as_ref().map(|a| a.credits_remaining.clone()).unwrap_or(BigDecimal::from(0)),
                new_balance: result.credits_remaining.clone(),
                action_source: "stripe_webhook_event".to_string(),
                action_type: "buy_credits".to_string(),
                entity_id: Some(result.id),
            },
        ).await?;

        log::info!(
            "Organization credit allocation transaction logged: organization_id={}, credits_added={}, previous_balance={}, new_balance={}",
            organization_id,
            credits_to_add,
            current_allocation.as_ref().map(|a| a.credits_remaining.clone()).unwrap_or(BigDecimal::from(0)),
            result.credits_remaining
        );
    }

    Ok(result)
}

