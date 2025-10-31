//! Query to get already deducted transactions by action source and entity IDs
//!
//! This query fetches credit transaction records that match the given criteria
//! to identify which transactions have already been processed and should be
//! filtered out from bulk processing.

use serde::{Deserialize, Serialize};
use sqlx::{Error, PgPool};
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

/// Record of an already deducted transaction
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeductedTransaction {
    /// Credit transaction record ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub record_id: Uuid,
    
    /// Entity ID that was deducted (e.g., transaction ID from external system)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub entity_id: Option<Uuid>,
}

/// Get already deducted transactions by action source and entity IDs
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `action_source` - Action source to filter by (e.g., "imageboard")
/// * `entity_ids` - List of entity IDs to check for existing deductions
/// * `action_type` - Optional action type to filter by (e.g., "transaction_deduction")
///
/// # Returns
/// Vector of deducted transaction records containing the credit transaction record ID
/// and the entity ID that was already processed.
///
/// # Purpose
/// This function is used to identify which transactions from a bulk request
/// have already been processed, allowing the system to skip them and avoid
/// duplicate credit deductions.
#[instrument(skip(pool))]
pub async fn get_deducted_transactions(
    pool: &PgPool,
    action_source: &str,
    entity_ids: &[Uuid],
    action_type: Option<&str>,
) -> Result<Vec<DeductedTransaction>, Error> {
    if entity_ids.is_empty() {
        return Ok(vec![]);
    }

    let deducted_transactions = if let Some(action_type_filter) = action_type {
        sqlx::query_as!(
            DeductedTransaction,
            r#"
            SELECT 
                id as record_id,
                entity_id
            FROM credit_transactions
            WHERE action_source = $1
              AND action_type = $2
              AND entity_id IS NOT NULL
              AND entity_id = ANY($3)
              AND credits_changed < 0
            ORDER BY created_at DESC
            "#,
            action_source,
            action_type_filter,
            entity_ids as &[Uuid]
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            DeductedTransaction,
            r#"
            SELECT 
                id as record_id,
                entity_id
            FROM credit_transactions
            WHERE action_source = $1
              AND entity_id IS NOT NULL
              AND entity_id = ANY($2)
              AND credits_changed < 0
            ORDER BY created_at DESC
            "#,
            action_source,
            entity_ids as &[Uuid]
        )
        .fetch_all(pool)
        .await?
    };

    Ok(deducted_transactions)
}
