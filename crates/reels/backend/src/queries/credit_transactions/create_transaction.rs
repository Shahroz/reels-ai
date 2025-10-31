//! Create credit transaction record
//!
//! This query creates a new credit transaction record for audit purposes.

use sqlx::{PgPool, Result};
use crate::db::credit_transaction::{CreditTransaction, DbCreditTransaction};
use crate::schemas::credit_transactions_schemas::CreateCreditTransactionParams;

/// Create a new credit transaction record
/// 
/// This function creates a transaction record for audit purposes when credits are
/// added, deducted, or modified. It returns the created transaction record.
#[allow(clippy::disallowed_methods)]
pub async fn create_credit_transaction(
    pool: &PgPool,
    params: CreateCreditTransactionParams,
) -> Result<CreditTransaction> {
    let transaction = sqlx::query_as::<_, DbCreditTransaction>(
        r#"
        INSERT INTO credit_transactions (
            user_id, organization_id, credits_changed, previous_balance, new_balance,
            action_source, action_type, entity_id, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, CURRENT_TIMESTAMP)
        RETURNING 
            id,
            user_id,
            organization_id,
            credits_changed,
            previous_balance,
            new_balance,
            action_source,
            action_type,
            entity_id,
            created_at
        "#,
    )
    .bind(params.user_id)
    .bind(params.organization_id)
    .bind(params.credits_changed)
    .bind(params.previous_balance)
    .bind(params.new_balance)
    .bind(params.action_source)
    .bind(params.action_type)
    .bind(params.entity_id)
    .fetch_one(pool)
    .await?;

    Ok(transaction.into())
}
