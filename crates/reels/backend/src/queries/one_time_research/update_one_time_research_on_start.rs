//! Updates a one-time research task when it starts execution.
//!
//! This function sets the task's status to 'running' and records the
//! start time. It is called by the internal task execution endpoint.

#[tracing::instrument(skip(pool))]
pub async fn update_one_time_research_on_start(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
) -> Result<crate::db::one_time_research::OneTimeResearch, sqlx::Error> {
    let query = sqlx::query_as!(
        crate::db::one_time_research::OneTimeResearch,
        r#"
        UPDATE one_time_researches
        SET status = 'running', started_at = NOW(), updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
        id
    );
    let updated_research = query.fetch_one(pool).await?;
    std::result::Result::Ok(updated_research)
}