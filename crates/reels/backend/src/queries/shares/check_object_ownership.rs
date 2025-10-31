//! Checks if a user is the direct owner of a given object.
//!
//! This function queries the database to determine if the `user_id` provided
//! is the owner of the object specified by `object_id` and `object_type`.
//! It supports various object types like styles, creatives, documents, etc.
//! Returns a boolean indicating ownership.

use sqlx::PgPool;
use uuid::Uuid;

pub async fn check_object_ownership(
    pool: &PgPool,
    user_id: Uuid,
    object_id: Uuid,
    object_type: &str,
) -> Result<bool, sqlx::Error> {
    let is_owner = match object_type {
        "style" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM styles WHERE id = $1 AND user_id = $2)", object_id, user_id)
                .fetch_one(pool).await?.unwrap_or(false)
        }
        "creative" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM creatives c JOIN collections col ON c.collection_id = col.id WHERE c.id = $1 AND col.user_id = $2)", object_id, user_id)
                .fetch_one(pool).await?.unwrap_or(false)
        }
        "document" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1 AND user_id = $2)", object_id, user_id)
                .fetch_one(pool).await?.unwrap_or(false)
        }
        "custom_format" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM custom_creative_formats WHERE id = $1 AND user_id = $2)", object_id, user_id)
                .fetch_one(pool).await?.unwrap_or(false)
        }
        "asset" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM assets WHERE id = $1 AND user_id = $2)", object_id, user_id)
                .fetch_one(pool).await?.unwrap_or(false)
        }
        "collection" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM collections WHERE id = $1 AND user_id = $2)", object_id, user_id)
                .fetch_one(pool).await?.unwrap_or(false)
        }
        // If the object type is not recognized, we cannot determine ownership.
        // It's safer to assume no ownership and let other permission checks proceed.
        _ => false,
    };
    Ok(is_owner)
}