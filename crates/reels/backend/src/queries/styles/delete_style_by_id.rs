//! Deletes a style record from the database.
//!
//! This function removes a style by its ID. It's the final step in the
//! deletion process after permissions have been checked and shares have been cleaned up.

use sqlx::{postgres::PgQueryResult, Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(skip(tx))]
pub async fn delete_style_by_id(
    tx: &mut Transaction<'_, Postgres>,
    style_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!("DELETE FROM styles WHERE id = $1", style_id)
        .execute(&mut **tx)
        .await
}

#[cfg(test)]
mod tests {}