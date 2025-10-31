//! Updates the status (is_enabled) of an infinite research task.
//!
//! This function toggles the `is_enabled` flag for a specific infinite research
//! task and returns the updated record.
//! Follows the one-item-per-file guideline.

#[tracing::instrument(skip(pool))]
pub async fn update_infinite_research_status(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    is_enabled: bool,
) -> Result<crate::db::infinite_research::InfiniteResearch, sqlx::Error> {
    let query = sqlx::query_as!(
        crate::db::infinite_research::InfiniteResearch,
        r#"
        UPDATE infinite_researches
        SET is_enabled = $1, updated_at = NOW()
        WHERE id = $2 AND user_id = $3
        RETURNING *
        "#,
        is_enabled,
        id,
        user_id
    );
    let updated_research = query.fetch_one(pool).await?;
    std::result::Result::Ok(updated_research)
}