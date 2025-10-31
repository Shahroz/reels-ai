//! Delete credit transactions by organization ID.
//!
//! This function deletes all credit transaction records associated with a given organization.
//! This is typically used as part of organization cleanup/deletion workflows.

use sqlx::Postgres;
use tracing::instrument;
use uuid::Uuid;

/// Delete all credit transactions for a specific organization
#[instrument(skip(executor))]
pub async fn delete_credit_transactions_by_organization<'a, E>(
    executor: E,
    organization_id: Uuid,
) -> Result<u64, sqlx::Error>
where
    E: sqlx::Executor<'a, Database = Postgres>,
{
    let result = sqlx::query!(
        r#"
        DELETE FROM credit_transactions 
        WHERE organization_id = $1
        "#,
        organization_id
    )
    .execute(executor)
    .await?;

    Ok(result.rows_affected())
}

