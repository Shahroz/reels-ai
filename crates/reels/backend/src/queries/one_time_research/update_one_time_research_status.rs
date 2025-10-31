//! Updates the status and cloud task name of a one-time research task.
//!
//! This function updates the `status` and `cloud_task_name` for a given
//! research task identified by its ID. This is typically used after a task
//! has been successfully queued in Google Cloud Tasks.

#[tracing::instrument(skip(pool))]
pub async fn update_one_time_research_status(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    status: &str,
    cloud_task_name: Option<&str>,
) -> Result<crate::db::one_time_research::OneTimeResearch, sqlx::Error> {
    let query = sqlx::query_as!(
        crate::db::one_time_research::OneTimeResearch,
        r#"
        UPDATE one_time_researches
        SET status = $1, cloud_task_name = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING *
        "#,
        status,
        cloud_task_name,
        id
    );
    let updated_research = query.fetch_one(pool).await?;
    std::result::Result::Ok(updated_research)
}