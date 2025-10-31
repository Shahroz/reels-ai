//! Updates a one-time research task when it finishes execution.
//!
//! This function updates the task's status to 'completed' or 'failed',
//! records the finish time, and stores either the output log URL on success
//! or an error message on failure.

#[tracing::instrument(skip(pool))]
pub async fn update_one_time_research_on_finish(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    output_log_url: Option<&str>,
    error_message: Option<&str>,
) -> Result<crate::db::one_time_research::OneTimeResearch, sqlx::Error> {
    let status = if error_message.is_some() { "failed" } else { "completed" };

    let query = sqlx::query_as!(
        crate::db::one_time_research::OneTimeResearch,
        r#"
        UPDATE one_time_researches
        SET
            status = $1,
            finished_at = NOW(),
            updated_at = NOW(),
            output_log_url = $2,
            error_message = $3
        WHERE id = $4
        RETURNING *
        "#,
        status,
        output_log_url,
        error_message,
        id
    );
    let updated_research = query.fetch_one(pool).await?;
    std::result::Result::Ok(updated_research)
}