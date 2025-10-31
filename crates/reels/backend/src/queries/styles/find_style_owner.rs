//! Fetches the owner's user ID for a given style.
//!
//! This is a lightweight query used for permission checks to quickly determine
//! if the requesting user is the direct owner of the style.

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

// Helper struct to fetch only user_id for permission check
#[derive(Copy, Clone, Debug)]
pub struct StyleOwner {
    pub user_id: Uuid,
}

#[tracing::instrument(skip(tx))]
pub async fn find_style_owner(
    tx: &mut Transaction<'_, Postgres>,
    style_id: Uuid,
) -> Result<Option<StyleOwner>, sqlx::Error> {
    sqlx::query_as!(StyleOwner, "SELECT user_id FROM styles WHERE id = $1", style_id)
        .fetch_optional(&mut **tx)
        .await
}

#[cfg(test)]
mod tests {}