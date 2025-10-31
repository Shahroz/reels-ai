#![allow(clippy::disallowed_methods)]
//! Delete user credit allocation.
//!
//! This function deletes a user credit allocation record.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

/// Delete user credit allocation
#[instrument(skip(pool))]
pub async fn delete_user_credit_allocation(pool: &PgPool, user_id: Uuid) -> Result<u64, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM user_credit_allocation 
        WHERE user_id = $1
        "#,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
