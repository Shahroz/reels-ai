//! Checks if a user exists by ID.
//!
//! This query function provides a simple boolean check for user existence without
//! fetching the full user record. Useful for validation in service layers before
//! performing operations that reference user IDs.

pub async fn user_exists(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    user_id: uuid::Uuid,
) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        r#"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as "exists!""#,
        user_id
    )
    .fetch_one(executor)
    .await?;
    
    Ok(result.exists)
}

