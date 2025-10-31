//! Creates a new share or updates an existing one.
//!
//! This function performs an "upsert" operation on the `object_shares` table.
//! If a share for the given object and entity already exists, it updates the
//! access level. Otherwise, it inserts a new share record.
//! Returns the created or updated `ObjectShare`.

use crate::db::shares::{AccessLevel, EntityType, ObjectShare};
use sqlx::Transaction;
use uuid::Uuid;

pub async fn upsert_share(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    object_id: Uuid,
    object_type: &str,
    entity_id: Uuid,
    entity_type: EntityType,
    access_level: AccessLevel,
) -> Result<ObjectShare, sqlx::Error> {
    sqlx::query_as!(
        ObjectShare,
        r#"
        INSERT INTO object_shares (object_id, object_type, entity_id, entity_type, access_level)
        VALUES ($1, $2, $3, $4::object_share_entity_type, $5::object_share_access_level)
        ON CONFLICT (object_id, object_type, entity_id, entity_type)
        DO UPDATE SET access_level = EXCLUDED.access_level, updated_at = NOW()
        RETURNING id, object_id, object_type, entity_id, entity_type AS "entity_type!: EntityType", access_level AS "access_level!: AccessLevel", created_at, updated_at,
        NULL::text AS entity_name
        "#,
        object_id,
        object_type,
        entity_id,
        entity_type as EntityType,
        access_level as AccessLevel
    )
    .fetch_one(&mut **tx)
    .await
}