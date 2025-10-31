//! Remove a prompt from user's favorites

use anyhow::{Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

/// Remove a favorite prompt by ID
/// Returns true if deleted, false if not found or not owned by user
pub async fn remove_favorite_prompt(
    pool: &PgPool,
    prompt_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let result = sqlx::query!(
        r#"
        DELETE FROM favorited_prompts
        WHERE id = $1 AND user_id = $2
        "#,
        prompt_id,
        user_id,
    )
    .execute(pool)
    .await
    .context("Failed to remove favorite prompt")?;
    
    Ok(result.rows_affected() > 0)
}

