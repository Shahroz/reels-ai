//! Defines the `count_bundles_for_user` database query function.
//!
//! This function counts all bundles for a specific user ID,
//! optionally filtering by a search pattern on the bundle name.
//! Adheres to the project's Rust coding standards.

pub async fn count_bundles_for_user(
    pool: &sqlx::PgPool,
    user_id: sqlx::types::Uuid,
    search: &str,
) -> std::result::Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM bundles
        WHERE user_id = $1 AND name ILIKE $2
        "#,
        user_id,
        search,
    )
    .fetch_one(pool)
    .await?;

    Ok(result.count)
}