#![allow(clippy::disallowed_methods)]
//! Admin-specific organization credit update with transaction logging.
//!
//! This function updates an organization's credits and creates a credit transaction
//! log for audit purposes. Unlike add/subtract operations, this sets an absolute value.
//! Used exclusively by admin endpoints to manually adjust organization credits.
//! Includes full transaction logging for financial audit trail.

/// Updates organization credits to a specific value with transaction logging
///
/// # Arguments
///
/// * `pool` - The database connection pool
/// * `organization_id` - The organization whose credits to update
/// * `new_credits` - The new absolute credit value
/// * `admin_user_id` - The admin user performing this operation
///
/// # Returns
///
/// The updated OrganizationCreditAllocation
#[tracing::instrument(skip(pool))]
pub async fn admin_update_organization_credits_with_transaction(
    pool: &sqlx::PgPool,
    organization_id: uuid::Uuid,
    new_credits: bigdecimal::BigDecimal,
    admin_user_id: uuid::Uuid,
) -> Result<crate::db::organization_credit_allocation::OrganizationCreditAllocation, sqlx::Error> {
    // Start a transaction
    let mut tx = pool.begin().await?;
    
    // Get current allocation to track the change
    let current_allocation = sqlx::query_as!(
        crate::db::organization_credit_allocation::OrganizationCreditAllocation,
        r#"
        SELECT id, organization_id, credits_remaining, last_reset_date, created_at, updated_at
        FROM organization_credit_allocation
        WHERE organization_id = $1
        FOR UPDATE
        "#,
        organization_id
    )
    .fetch_optional(&mut *tx)
    .await?;
    
    let previous_balance = current_allocation
        .as_ref()
        .map(|a| a.credits_remaining.clone())
        .unwrap_or_else(|| bigdecimal::BigDecimal::from(0));
    
    // Calculate the change amount
    let credits_changed = new_credits.clone() - previous_balance.clone();
    
    // Update or create the credit allocation
    let updated_allocation = if current_allocation.is_some() {
        // Update existing allocation
        sqlx::query_as!(
            crate::db::organization_credit_allocation::OrganizationCreditAllocation,
            r#"
            UPDATE organization_credit_allocation
            SET credits_remaining = $1, updated_at = CURRENT_TIMESTAMP
            WHERE organization_id = $2
            RETURNING id, organization_id, credits_remaining, last_reset_date, created_at, updated_at
            "#,
            new_credits,
            organization_id
        )
        .fetch_one(&mut *tx)
        .await?
    } else {
        // Create new allocation if it doesn't exist
        sqlx::query_as!(
            crate::db::organization_credit_allocation::OrganizationCreditAllocation,
            r#"
            INSERT INTO organization_credit_allocation (organization_id, credits_remaining, last_reset_date)
            VALUES ($1, $2, NOW())
            RETURNING id, organization_id, credits_remaining, last_reset_date, created_at, updated_at
            "#,
            organization_id,
            new_credits
        )
        .fetch_one(&mut *tx)
        .await?
    };
    
    // Create credit transaction log
    let _transaction_log = sqlx::query!(
        r#"
        INSERT INTO credit_transactions (
            user_id,
            organization_id,
            credits_changed,
            previous_balance,
            new_balance,
            action_source,
            action_type,
            entity_id,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
        RETURNING id
        "#,
        admin_user_id,
        organization_id,
        credits_changed,
        previous_balance,
        new_credits,
        "admin",
        "manual_adjustment",
        organization_id
    )
    .fetch_one(&mut *tx)
    .await?;
    
    // Commit the transaction
    tx.commit().await?;
    
    log::info!(
        "Admin {} updated organization {} credits from {} to {}",
        admin_user_id,
        organization_id,
        previous_balance,
        new_credits
    );
    
    Ok(updated_allocation)
}

