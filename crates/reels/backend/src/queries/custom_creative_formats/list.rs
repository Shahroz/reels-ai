//! Fetches a paginated list of custom creative formats from the database.
//!
//! This function retrieves custom creative formats with sorting, filtering,
//! and pagination. It supports filtering by user ID or public status, searching
//! by name, and sorting by various fields.
//! Adheres to one item per file guideline.

use crate::db::custom_creative_formats::CustomCreativeFormat;
use sqlx::PgPool;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub async fn list_formats(
    pool: &PgPool,
    user_id: Uuid,
    is_public: bool,
    search: Option<String>,
    sort_by: &str,
    sort_order: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<CustomCreativeFormat>, sqlx::Error> {
    let search_pattern = search.map(|s| format!("%{s}%")).unwrap_or_else(|| "%".into());

    let items = match (sort_by, sort_order) {
        ("name", "asc") => {
            if is_public {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE (is_public = TRUE AND user_id IS NULL) AND name ILIKE $1 ORDER BY name asc LIMIT $2 OFFSET $3"#,
                    &search_pattern, limit, offset
                ).fetch_all(pool).await
            } else {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE user_id = $1 AND name ILIKE $2 ORDER BY name asc LIMIT $3 OFFSET $4"#,
                    user_id, &search_pattern, limit, offset
                ).fetch_all(pool).await
            }
        }
        ("name", "desc") => {
            if is_public {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE (is_public = TRUE AND user_id IS NULL) AND name ILIKE $1 ORDER BY name desc LIMIT $2 OFFSET $3"#,
                    &search_pattern, limit, offset
                ).fetch_all(pool).await
            } else {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE user_id = $1 AND name ILIKE $2 ORDER BY name desc LIMIT $3 OFFSET $4"#,
                    user_id, &search_pattern, limit, offset
                ).fetch_all(pool).await
            }
        }
        ("created_at", "asc") => {
            if is_public {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE (is_public = TRUE AND user_id IS NULL) AND name ILIKE $1 ORDER BY created_at asc LIMIT $2 OFFSET $3"#,
                    &search_pattern, limit, offset
                ).fetch_all(pool).await
            } else {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE user_id = $1 AND name ILIKE $2 ORDER BY created_at asc LIMIT $3 OFFSET $4"#,
                    user_id, &search_pattern, limit, offset
                ).fetch_all(pool).await
            }
        }
        ("created_at", "desc") => { // Default case
            if is_public {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE (is_public = TRUE AND user_id IS NULL) AND name ILIKE $1 ORDER BY created_at desc LIMIT $2 OFFSET $3"#,
                    &search_pattern, limit, offset
                ).fetch_all(pool).await
            } else {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE user_id = $1 AND name ILIKE $2 ORDER BY created_at desc LIMIT $3 OFFSET $4"#,
                    user_id, &search_pattern, limit, offset
                ).fetch_all(pool).await
            }
        }
        ("updated_at", "asc") => {
            if is_public {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE (is_public = TRUE AND user_id IS NULL) AND name ILIKE $1 ORDER BY updated_at asc LIMIT $2 OFFSET $3"#,
                    &search_pattern, limit, offset
                ).fetch_all(pool).await
            } else {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE user_id = $1 AND name ILIKE $2 ORDER BY updated_at asc LIMIT $3 OFFSET $4"#,
                    user_id, &search_pattern, limit, offset
                ).fetch_all(pool).await
            }
        }
        ("updated_at", "desc") => {
            if is_public {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE (is_public = TRUE AND user_id IS NULL) AND name ILIKE $1 ORDER BY updated_at desc LIMIT $2 OFFSET $3"#,
                    &search_pattern, limit, offset
                ).fetch_all(pool).await
            } else {
                sqlx::query_as!(CustomCreativeFormat,
                    r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE user_id = $1 AND name ILIKE $2 ORDER BY updated_at desc LIMIT $3 OFFSET $4"#,
                    user_id, &search_pattern, limit, offset
                ).fetch_all(pool).await
            }
        }
        _ => unreachable!(),
    };
    items
}
