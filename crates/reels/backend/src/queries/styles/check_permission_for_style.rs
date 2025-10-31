//! Checks if a user has a specific level of access to a style.
//!
//! This function verifies if a a `style` object has been shared with a user
//! directly or through an organization they belong to, with at least one of
//! the specified access levels.

use crate::db::shares::{AccessLevel, EntityType};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(skip(tx))]
pub async fn check_permission_for_style(
    tx: &mut Transaction<'_, Postgres>,
    style_id: Uuid,
    user_id: Uuid,
    org_ids: &[Uuid],
    required_access_levels: &[String],
) -> Result<bool, sqlx::Error> {
    let sql_object_type = "style".to_string();

    let has_permission = sqlx::query_scalar!(
        r#"
            SELECT EXISTS (
                SELECT 1 FROM object_shares
                WHERE object_id = $1 AND object_type = $2
                AND (
                    (entity_type = $3 AND entity_id = $4) OR
                    (entity_type = $5 AND entity_id = ANY($6))
                )
                AND access_level::text = ANY($7) -- Check for any of the required levels
            )
        "#,
        style_id,
        sql_object_type,
        EntityType::User as EntityType,
        user_id,
        EntityType::Organization as EntityType,
        org_ids,
        required_access_levels
    )
    .fetch_one(&mut **tx)
    .await?
    .unwrap_or(false);

    Ok(has_permission)
}

#[cfg(test)]
mod tests {
    // Tests for query functions would typically involve setting up a test database
    // and mocking data, which is beyond the scope of this refactoring.
}
