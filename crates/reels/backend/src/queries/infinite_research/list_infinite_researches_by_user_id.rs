// Lists all infinite research tasks for a specific user, including last execution status.
//
// This function retrieves all infinite research tasks for a user, joining with
// the executions table to get the ID, start time, and status of the most recent
// execution for each task.

#[tracing::instrument(skip(pool))]
pub async fn list_infinite_researches_by_user_id(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> Result<std::vec::Vec<crate::db::infinite_research_list_item::InfiniteResearchListItem>, sqlx::Error> {
    let query = sqlx::query_as!(
        crate::db::infinite_research_list_item::InfiniteResearchListItem,
        r#"
        WITH last_execution AS (
            SELECT
                infinite_research_id,
                id,
                started_at,
                status,
                ROW_NUMBER() OVER(PARTITION BY infinite_research_id ORDER BY started_at DESC) as rn
            FROM infinite_research_executions
        )
        SELECT
            ir.id,
            ir.user_id,
            ir.name,
            ir.prompt,
            ir.cron_schedule,
            ir.is_enabled,
            ir.scheduler_job_name,
            ir.created_at,
            ir.updated_at,
            le.id as "last_execution_id?",
            le.started_at as "last_execution_started_at?",
            le.status as "last_execution_status?"
        FROM infinite_researches ir
        LEFT JOIN last_execution le ON ir.id = le.infinite_research_id AND le.rn = 1
        WHERE ir.user_id = $1
        ORDER BY ir.updated_at DESC
        "#,
        user_id
    );
    let researches = query.fetch_all(pool).await?;
    std::result::Result::Ok(researches)
}
