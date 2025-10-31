//! Defines database query functions for the `requests` table.
//!
//! This module encapsulates all direct database interactions for request records,
//! following the one-item-per-file and FQN guidelines where applicable for query logic.
//! Functions here are designed to be called from route handlers or other business logic.

use anyhow::Context;
use sqlx::{types::Uuid, PgPool};
use tracing::instrument;

/// Fetches a single request record by its ID for a specific user.
#[instrument(skip(pool))]
pub async fn get_request_by_id_and_user_id(
    pool: &PgPool,
    request_id: i32,
    user_id: Uuid,
) -> anyhow::Result<Option<crate::db::requests::RequestRecord>> {
    sqlx::query_as!(
        crate::db::requests::RequestRecord,
        "SELECT * FROM requests WHERE id = $1 AND user_id = $2",
        request_id,
        user_id
    )
    .fetch_optional(pool)
    .await
    .context("Failed to fetch request by ID from database")
}

/// Deletes a request record by its ID for a specific user.
/// Returns the number of rows affected.
#[instrument(skip(pool))]
pub async fn delete_request_by_id_and_user_id(
    pool: &PgPool,
    request_id: i32,
    user_id: Uuid,
) -> anyhow::Result<u64> {
    let result = sqlx::query!(
        "DELETE FROM requests WHERE id = $1 AND user_id = $2",
        request_id,
        user_id
    )
    .execute(pool)
    .await
    .context("Failed to delete request from database")?;
    Ok(result.rows_affected())
}

/// Counts the total number of requests for a user, with an optional search filter.
#[instrument(skip(pool))]
pub async fn count_requests_for_user(
    pool: &PgPool,
    user_id: Uuid,
    search_pattern: &str,
    search_term: &str,
) -> anyhow::Result<i64> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM requests
           WHERE user_id = $1 AND deleted_at IS NULL
           AND (what_to_create ILIKE $2 OR $3 = '')"#,
        user_id,
        search_pattern,
        search_term
    )
    .fetch_one(pool)
    .await
    .context("Failed to count requests in database")?;
    Ok(count.unwrap_or(0))
}

/// Lists requests for a user with pagination, sorting, and search.
#[instrument(skip(pool))]
#[allow(clippy::too_many_arguments)]
pub async fn list_requests_for_user(
    pool: &PgPool,
    user_id: Uuid,
    search_pattern: &str,
    search_term: &str,
    sort_by: &str,
    sort_order: &str,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<crate::db::requests::RequestRecord>> {
    let items_result = match (sort_by, sort_order) {
        ("created_at", "asc") => {
            sqlx::query_as!(
                crate::db::requests::RequestRecord,
                r#"SELECT * FROM requests
                   WHERE user_id = $1 AND deleted_at IS NULL AND (what_to_create ILIKE $2 OR $3 = '')
                   ORDER BY created_at asc NULLS LAST
                   LIMIT $4 OFFSET $5"#,
                user_id, search_pattern, search_term, limit, offset
            )
            .fetch_all(pool)
            .await
        }
        ("created_at", "desc") => {
            sqlx::query_as!(
                crate::db::requests::RequestRecord,
                r#"SELECT * FROM requests
                   WHERE user_id = $1 AND deleted_at IS NULL AND (what_to_create ILIKE $2 OR $3 = '')
                   ORDER BY created_at desc NULLS LAST
                   LIMIT $4 OFFSET $5"#,
                user_id, search_pattern, search_term, limit, offset
            )
            .fetch_all(pool)
            .await
        }
        ("status", "asc") => {
            sqlx::query_as!(
                crate::db::requests::RequestRecord,
                r#"SELECT * FROM requests
                   WHERE user_id = $1 AND deleted_at IS NULL AND (what_to_create ILIKE $2 OR $3 = '')
                   ORDER BY status asc NULLS LAST
                   LIMIT $4 OFFSET $5"#,
                user_id, search_pattern, search_term, limit, offset
            )
            .fetch_all(pool)
            .await
        }
        ("status", "desc") => {
            sqlx::query_as!(
                crate::db::requests::RequestRecord,
                r#"SELECT * FROM requests
                   WHERE user_id = $1 AND deleted_at IS NULL AND (what_to_create ILIKE $2 OR $3 = '')
                   ORDER BY status desc NULLS LAST
                   LIMIT $4 OFFSET $5"#,
                user_id, search_pattern, search_term, limit, offset
            )
            .fetch_all(pool)
            .await
        }
        ("what_to_create", "asc") => {
            sqlx::query_as!(
                crate::db::requests::RequestRecord,
                r#"SELECT * FROM requests
                   WHERE user_id = $1 AND deleted_at IS NULL AND (what_to_create ILIKE $2 OR $3 = '')
                   ORDER BY what_to_create asc NULLS LAST
                   LIMIT $4 OFFSET $5"#,
                user_id, search_pattern, search_term, limit, offset
            )
            .fetch_all(pool)
            .await
        }
        ("what_to_create", "desc") => {
            sqlx::query_as!(
                crate::db::requests::RequestRecord,
                r#"SELECT * FROM requests
                   WHERE user_id = $1 AND deleted_at IS NULL AND (what_to_create ILIKE $2 OR $3 = '')
                   ORDER BY what_to_create desc NULLS LAST
                   LIMIT $4 OFFSET $5"#,
                user_id, search_pattern, search_term, limit, offset
            )
            .fetch_all(pool)
            .await
        }
        ("finished_at", "asc") => {
            sqlx::query_as!(
                crate::db::requests::RequestRecord,
                r#"SELECT * FROM requests
                   WHERE user_id = $1 AND deleted_at IS NULL AND (what_to_create ILIKE $2 OR $3 = '')
                   ORDER BY finished_at asc NULLS LAST
                   LIMIT $4 OFFSET $5"#,
                user_id, search_pattern, search_term, limit, offset
            )
            .fetch_all(pool)
            .await
        }
        ("finished_at", "desc") => {
            sqlx::query_as!(
                crate::db::requests::RequestRecord,
                r#"SELECT * FROM requests
                   WHERE user_id = $1 AND deleted_at IS NULL AND (what_to_create ILIKE $2 OR $3 = '')
                   ORDER BY finished_at desc NULLS LAST
                   LIMIT $4 OFFSET $5"#,
                user_id, search_pattern, search_term, limit, offset
            )
            .fetch_all(pool)
            .await
        }
        _ => unreachable!("Validated sort_by ('{}') and sort_order ('{}') should cover all cases", sort_by, sort_order),
    };
    items_result.context("Failed to list requests from database")
}