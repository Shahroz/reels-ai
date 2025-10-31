use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Error, PgPool};
use uuid::Uuid;
use utoipa::ToSchema;
use tracing::instrument;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResearchWorkflow {
    pub id: i32,
    pub name: String,
    #[schema(format = "uuid", value_type=String)]
    pub user_id: Uuid,
    #[schema(value_type = Object)]
    pub payload: serde_json::Value,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateResearchWorkflow {
    pub name: String,
    pub user_id: Uuid,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResearchWorkflow {
    pub name: Option<String>,
    pub payload: Option<serde_json::Value>,
}

#[instrument(skip(pool, workflow))]
pub async fn create_workflow(
    pool: &PgPool,
    workflow: CreateResearchWorkflow,
) -> Result<ResearchWorkflow, Error> {
    sqlx::query_as!(
        ResearchWorkflow,
        r#"
        INSERT INTO research_workflows (name, user_id, payload)
        VALUES ($1, $2, $3)
        RETURNING id, name, user_id, payload, created_at, updated_at
        "#,
        workflow.name,
        workflow.user_id,
        workflow.payload,
    )
    .fetch_one(pool)
    .await
}

#[instrument(skip(pool))]
pub async fn get_workflow(
    pool: &PgPool,
    id: i32,
    user_id: Uuid,
) -> Result<ResearchWorkflow, Error> {
    sqlx::query_as!(
        ResearchWorkflow,
        r#"
        SELECT id, name, user_id, payload, created_at, updated_at
        FROM research_workflows
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id,
    )
    .fetch_one(pool)
    .await
}

#[instrument(skip(pool))]
pub async fn list_workflows(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<ResearchWorkflow>, Error> {
    sqlx::query_as!(
        ResearchWorkflow,
        r#"
        SELECT id, name, user_id, payload, created_at, updated_at
        FROM research_workflows
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        OFFSET $3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
}

#[instrument(skip(pool, workflow))]
pub async fn update_workflow(
    pool: &PgPool,
    id: i32,
    user_id: Uuid,
    workflow: UpdateResearchWorkflow,
) -> Result<ResearchWorkflow, Error> {
    sqlx::query_as!(
        ResearchWorkflow,
        r#"
        UPDATE research_workflows
        SET 
            name = COALESCE($1, name),
            payload = COALESCE($2, payload),
            updated_at = NOW()
        WHERE id = $3 AND user_id = $4
        RETURNING id, name, user_id, payload, created_at, updated_at
        "#,
        workflow.name,
        workflow.payload,
        id,
        user_id
    )
    .fetch_one(pool)
    .await
}

#[instrument(skip(pool))]
pub async fn delete_workflow(pool: &PgPool, id: i32, user_id: Uuid) -> Result<(), Error> {
    sqlx::query!(
        r#"
        DELETE FROM research_workflows
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

