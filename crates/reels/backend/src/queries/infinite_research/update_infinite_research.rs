//! Updates an existing infinite research task.
//!
//! This function modifies the details of an existing infinite research task
//! identified by its ID and user ID. It returns the updated task.
//! Follows the one-item-per-file guideline.

#[tracing::instrument(skip(pool, prompt))]
pub async fn update_infinite_research(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    name: &str,
    prompt: &str,
    cron_schedule: &str,
    is_enabled: bool,
    scheduler_job_name: Option<&str>,
) -> Result<crate::db::infinite_research::InfiniteResearch, sqlx::Error> {
    let query = sqlx::query_as!(
        crate::db::infinite_research::InfiniteResearch,
        r#"
        UPDATE infinite_researches
        SET name = $1, prompt = $2, cron_schedule = $3, is_enabled = $4, scheduler_job_name = $5, updated_at = NOW()
        WHERE id = $6 AND user_id = $7
        RETURNING *
        "#,
        name,
        prompt,
        cron_schedule,
        is_enabled,
        scheduler_job_name,
        id,
        user_id
    );
    let updated_research = query.fetch_one(pool).await?;
    std::result::Result::Ok(updated_research)
}