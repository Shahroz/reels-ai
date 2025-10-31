//! Creates a new infinite research task in the database.
//!
//! This function inserts a new record into the `infinite_researches` table
//! with the provided details. It returns the newly created task upon success.
//! Follows the one-item-per-file guideline.

#[tracing::instrument(skip(pool, prompt))]
pub async fn create_infinite_research(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    name: &str,
    prompt: &str,
    cron_schedule: &str,
    scheduler_job_name: Option<&str>,
) -> Result<crate::db::infinite_research::InfiniteResearch, sqlx::Error> {
    let query = sqlx::query_as!(
        crate::db::infinite_research::InfiniteResearch,
        r#"
        INSERT INTO infinite_researches (user_id, name, prompt, cron_schedule, scheduler_job_name)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        user_id,
        name,
        prompt,
        cron_schedule,
        scheduler_job_name
    );
    let infinite_research = query.fetch_one(pool).await?;
    std::result::Result::Ok(infinite_research)
}