//! Deletes an organization by its ID.
//!
//! This function handles cascading deletion of organization-related data including:
//! - Credit transactions associated with the organization
//! - The organization record itself
//!
//! All deletions are performed within a transaction to ensure data consistency.

use crate::queries::credit_transactions::delete_credit_transactions_by_organization;
use sqlx::{types::Uuid, PgPool};
use tracing::instrument;

/// Deletes an organization by its ID.
/// This function assumes all pre-checks (like ensuring no owned objects) have been performed.
/// It handles cascading deletion of credit transactions before removing the organization.
#[instrument(skip(pool))]
pub async fn delete_organization_by_id(pool: &PgPool, org_id: Uuid) -> anyhow::Result<u64> {
    // Start a transaction to ensure all deletions happen atomically
    let mut tx = pool.begin().await?;
    
    // 1. Delete all credit transactions associated with this organization
    let deleted_transactions = delete_credit_transactions_by_organization(&mut *tx, org_id).await?;
    log::info!(
        "Deleted {} credit transaction(s) for organization {}",
        deleted_transactions,
        org_id
    );
    
    // 2. Delete the organization itself
    let result = sqlx::query("DELETE FROM organizations WHERE id = $1")
        .bind(org_id)
        .execute(&mut *tx)
        .await?;
    
    // Commit the transaction
    tx.commit().await?;
    
    Ok(result.rows_affected())
} 