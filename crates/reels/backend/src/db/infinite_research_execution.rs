//! Manages database operations for the `infinite_research_executions` table.
//!
//! This module provides functions for creating and listing execution records
//! for infinite research tasks. Each execution represents a single run of a
//! scheduled research job.

/// Represents a single execution of an infinite research task.
#[derive(sqlx::FromRow, serde::Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct InfiniteResearchExecution {
    /// The unique identifier for this execution.
    #[schema(value_type = String, format = "uuid")]
    pub id: uuid::Uuid,
    /// The ID of the parent infinite research task.
    #[schema(value_type = String, format = "uuid")]
    pub infinite_research_id: uuid::Uuid,
    /// The current status of the execution (e.g., "running", "completed", "failed").
    pub status: std::string::String,
    /// The timestamp when the execution started.
    #[schema(value_type = String, format = "date-time")]
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// The timestamp when the execution finished.
    #[schema(value_type = String, format = "date-time", nullable = true)]
    pub finished_at: std::option::Option<chrono::DateTime<chrono::Utc>>,
   /// A JSON object containing output logs or results from the execution.
   #[schema(value_type = Object, nullable = true)]
   pub output_log: std::option::Option<std::string::String>,
   /// Any error message if the execution failed.
   #[schema(nullable = true)]
   pub error_message: std::option::Option<std::string::String>,
}

/// Creates a new execution record for an infinite research task.
#[tracing::instrument(skip(pool))]
pub async fn create_execution(
    pool: &sqlx::PgPool,
    infinite_research_id: uuid::Uuid,
    status: &str,
) -> Result<InfiniteResearchExecution, sqlx::Error> {
   let query = sqlx::query_as!(
       InfiniteResearchExecution,
       r#"
        INSERT INTO infinite_research_executions (infinite_research_id, status, output_log)
        VALUES ($1, $2, NULL)
        RETURNING *
        "#,
        infinite_research_id,
        status
    );
    let execution = query.fetch_one(pool).await?;
    std::result::Result::Ok(execution)
}

/// Updates the status and result of an execution upon completion.
#[tracing::instrument(skip(pool, output_log, error_message))]
pub async fn update_execution_on_finish(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    status: &str,
    output_log: std::option::Option<std::string::String>,
    error_message: std::option::Option<std::string::String>,
) -> Result<InfiniteResearchExecution, sqlx::Error> {
    let query = sqlx::query_as!(
        InfiniteResearchExecution,
        r#"
        UPDATE infinite_research_executions
        SET status = $1, output_log = $2, error_message = $3, finished_at = NOW()
        WHERE id = $4
        RETURNING *
        "#,
        status,
        output_log,
        error_message,
        id
    );
    let execution = query.fetch_one(pool).await?;
    std::result::Result::Ok(execution)
}

/// Lists all executions for a given infinite research task.
#[tracing::instrument(skip(pool))]
pub async fn list_executions_by_research_id(
    pool: &sqlx::PgPool,
    infinite_research_id: uuid::Uuid,
) -> Result<std::vec::Vec<InfiniteResearchExecution>, sqlx::Error> {
    let query = sqlx::query_as!(
        InfiniteResearchExecution,
        r#"
        SELECT * FROM infinite_research_executions
        WHERE infinite_research_id = $1
        ORDER BY started_at DESC
        "#,
        infinite_research_id
    );
    let executions = query.fetch_all(pool).await?;
   std::result::Result::Ok(executions)
}

/// Fetches an execution by its ID, ensuring it belongs to the specified user.
#[tracing::instrument(skip(pool))]
pub async fn get_execution_by_id_and_user_id(
    pool: &sqlx::PgPool,
    execution_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<std::option::Option<InfiniteResearchExecution>, sqlx::Error> {
    let execution = sqlx::query_as!(
        InfiniteResearchExecution,
        r#"
        SELECT exec.*
        FROM infinite_research_executions AS exec
        JOIN infinite_researches AS research ON exec.infinite_research_id = research.id
        WHERE exec.id = $1 AND research.user_id = $2
        "#,
        execution_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    std::result::Result::Ok(execution)
}