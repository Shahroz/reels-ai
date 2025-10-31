#![allow(clippy::disallowed_methods)]
//! Deduct credits from organization's remaining credits.
//!
//! This function ensures credits_remaining never goes below 0.
//! Uses a transaction to prevent race conditions and ensures atomicity.

use bigdecimal::BigDecimal;
use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::db::organization_credit_allocation::OrganizationCreditAllocation;

/// Deduct credits from organization's remaining credits
/// Returns both current and updated allocations for transaction logging
#[instrument(skip(pool))]
pub async fn deduct_organization_credits(
    pool: &PgPool,
    organization_id: Uuid,
    credits_to_deduct: BigDecimal,
) -> Result<(OrganizationCreditAllocation, OrganizationCreditAllocation), Error> {
    // Validate input - ensure we're not trying to deduct negative credits
    if credits_to_deduct < BigDecimal::from(0) {
        return Err(Error::Protocol("Cannot deduct negative credits".into()));
    }

    // Using a transaction to ensure atomicity and prevent race conditions
    let mut transaction = pool.begin().await?;

    // First, get the current credit allocation to check available credits
    let current_allocation = sqlx::query_as!(
        OrganizationCreditAllocation,
        r#"
        SELECT id, organization_id, credits_remaining, last_reset_date, created_at, updated_at
        FROM organization_credit_allocation
        WHERE organization_id = $1
        FOR UPDATE
        "#,
        organization_id
    )
    .fetch_optional(&mut *transaction)
    .await?;

    let current_allocation = current_allocation
        .ok_or_else(|| Error::RowNotFound)?;

    // Check if organization has enough credits
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
        OrganizationCreditAllocation,
        r#"
        UPDATE organization_credit_allocation 
        SET credits_remaining = $1, updated_at = CURRENT_TIMESTAMP
        WHERE organization_id = $2
        RETURNING id, organization_id, credits_remaining, last_reset_date, created_at, updated_at
        "#,
        new_credits_remaining,
        organization_id
    )
    .fetch_one(&mut *transaction)
    .await?;

    // Commit the transaction
    transaction.commit().await?;

    // Return both current and updated allocations
    Ok((current_allocation, updated_allocation))
}

