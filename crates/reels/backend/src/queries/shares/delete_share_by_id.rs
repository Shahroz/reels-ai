//! Deletes an object share by its unique ID.
//!
//! Executes a DELETE statement on the `object_shares` table for the given `share_id`.
//! Returns the number of rows affected, which should be 1 on success.
//! This is the final step in the share deletion process.

use sqlx::Transaction;
use uuid::Uuid;

pub async fn delete_share_by_id(
   tx: &mut Transaction<'_, sqlx::Postgres>,
   share_id: Uuid,
) -> Result<u64, sqlx::Error> {
   let result = sqlx::query!("DELETE FROM object_shares WHERE id = $1", share_id).execute(&mut **tx).await?;
   Ok(result.rows_affected())
}