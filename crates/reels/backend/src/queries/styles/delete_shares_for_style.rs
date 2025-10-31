//! Deletes all sharing records for a specific style.
//!
//! This is used during the style deletion process to clean up any `object_shares`
//! associated with the style being removed.

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(skip(tx))]
pub async fn delete_shares_for_style(
    tx: &mut Transaction<'_, Postgres>,
    style_id: Uuid,
) -> Result<(), sqlx::Error> {
    let sql_object_type_for_delete = "style";
    sqlx::query!(
        "DELETE FROM object_shares WHERE object_id = $1 AND object_type = $2",
        style_id,
        sql_object_type_for_delete
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {}