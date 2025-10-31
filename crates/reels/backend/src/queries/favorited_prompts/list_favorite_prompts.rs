//! List user's favorite prompts

use anyhow::{Context, Result};
use sqlx::PgPool;
use uuid::Uuid;
use crate::db::favorited_prompts::FavoritedPrompt;

/// List all favorite prompts for a user
/// Ordered by creation date (newest first)
pub async fn list_favorite_prompts(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<FavoritedPrompt>> {
    let prompts = sqlx::query_as!(
        FavoritedPrompt,
        r#"
        SELECT id, user_id, prompt_text, title, created_at
        FROM favorited_prompts
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .context("Failed to list favorite prompts")?;
    
    Ok(prompts)
}

