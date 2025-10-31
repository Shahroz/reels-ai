//! Checks if an organization exists by ID.
//!
//! This query function provides a simple boolean check for organization existence
//! without fetching the full organization record. Useful for validation in service
//! layers before performing operations that reference organization IDs.

pub async fn organization_exists(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    organization_id: uuid::Uuid,
) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        r#"SELECT EXISTS(SELECT 1 FROM organizations WHERE id = $1) as "exists!""#,
        organization_id
    )
    .fetch_one(executor)
    .await?;
    
    Ok(result.exists)
}

