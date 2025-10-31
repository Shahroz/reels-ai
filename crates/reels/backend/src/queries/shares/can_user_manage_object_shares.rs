//! Checks if a user has permission to manage shares for a given object.
//!
//! A user can manage shares if they are the direct owner of the object,
//! or if they have been granted 'editor' level access via a share.
//! This function composes ownership and editor-share checks.
//! Returns a boolean indicating management permission.

use crate::queries::shares::{
    check_object_ownership::check_object_ownership,
    check_user_has_editor_share::check_user_has_editor_share,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn can_user_manage_object_shares(
    pool: &PgPool,
    user_id: Uuid,
    object_id: Uuid,
    object_type: &str,
) -> Result<bool, sqlx::Error> {
    let is_owner = check_object_ownership(pool, user_id, object_id, object_type).await?;
    if is_owner {
        return Ok(true);
    }

    check_user_has_editor_share(pool, user_id, object_id, object_type).await
}