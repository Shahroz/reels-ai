//! Add a prompt to user's favorites

use anyhow::{Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

/// Arguments for adding a favorite prompt
pub struct AddFavoritePromptArgs {
    pub user_id: Uuid,
    pub prompt_text: String,
    pub title: Option<String>,
}

/// Add a prompt to user's favorites
/// Returns the favorited prompt ID on success
/// If the prompt is already favorited, returns existing ID (idempotent)
pub async fn add_favorite_prompt(
    pool: &PgPool,
    args: AddFavoritePromptArgs,
) -> Result<Uuid> {
    // Validate prompt is not empty
    let trimmed = args.prompt_text.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Prompt text cannot be empty");
    }
    
    // Insert or return existing (ON CONFLICT ... DO NOTHING + SELECT)
    // This makes the operation idempotent
    let result = sqlx::query_scalar!(
        r#"
        WITH inserted AS (
            INSERT INTO favorited_prompts (user_id, prompt_text, title)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, prompt_text) DO NOTHING
            RETURNING id
        )
        SELECT id FROM inserted
        UNION ALL
        SELECT id FROM favorited_prompts
        WHERE user_id = $1 AND prompt_text = $2
        LIMIT 1
        "#,
        args.user_id,
        trimmed,
        args.title,
    )
    .fetch_one(pool)
    .await
    .context("Failed to add favorite prompt")?;
    
    result.ok_or_else(|| anyhow::anyhow!("Failed to get prompt ID after insert"))
}

