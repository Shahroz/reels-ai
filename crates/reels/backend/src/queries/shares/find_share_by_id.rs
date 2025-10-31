//! Finds a single object share by its unique ID.
//!
//! Queries the `object_shares` table for a record matching the given `share_id`.
//! Returns an `Option<ObjectShare>` if a share is found.
//! This is used when deleting a share to verify its existence and get details.

use crate::db::shares::{AccessLevel, EntityType, ObjectShare};
use sqlx::Transaction;
use uuid::Uuid;

pub async fn find_share_by_id(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    share_id: Uuid,
) -> Result<Option<ObjectShare>, sqlx::Error> {
    sqlx::query_as!(
        ObjectShare,
        r#"SELECT
            id, object_id, object_type, entity_id, entity_type AS "entity_type!: EntityType",
            access_level AS "access_level!: AccessLevel", created_at, updated_at,
            NULL::text AS entity_name
           FROM object_shares WHERE id = $1"#,
       share_id
   )
   .fetch_optional(&mut *tx)
   .await
}