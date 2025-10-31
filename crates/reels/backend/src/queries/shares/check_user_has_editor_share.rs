//! Checks if a user has editor-level access to an object via a share.
//!
//! This function verifies if the user has an 'editor' share directly,
//! or is part of an organization that has an 'editor' share for the object.
//! It requires fetching the user's active organization memberships.
//! Returns a boolean indicating editor-level access.

use crate::db::shares::AccessLevel;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn check_user_has_editor_share(
    pool: &PgPool,
    user_id: Uuid,
    object_id: Uuid,
    object_type: &str,
) -> Result<bool, sqlx::Error> {
    // This function depends on `find_active_memberships_for_user`.
    // This function is not in the provided context but is assumed to exist in `db::organization_members`.
    let org_memberships = crate::db::organization_members::find_active_memberships_for_user(pool, user_id).await?;
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
    let org_ids_slice: &[Uuid] = if org_ids.is_empty() { &[] } else { &org_ids };

    let has_editor_share = sqlx::query_scalar!(
        r#"SELECT EXISTS (
            SELECT 1 FROM object_shares
            WHERE object_id = $1 AND object_type = $2 AND access_level = $3
            AND (
                (entity_type = 'user'::object_share_entity_type AND entity_id = $4) OR
                (entity_type = 'organization'::object_share_entity_type AND entity_id = ANY($5))
            )
        )"#,
        object_id,
        object_type,
        AccessLevel::Editor as AccessLevel,
        user_id,
        org_ids_slice
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    Ok(has_editor_share)
}