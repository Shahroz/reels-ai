use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use tracing::instrument;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ResearchWorkflow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub steps: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateResearchWorkflow {
    pub name: String,
    pub description: Option<String>,
    pub steps: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateResearchWorkflow {
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps: Option<serde_json::Value>,
}

#[instrument(skip(pool))]
pub async fn get_research_workflows(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<ResearchWorkflow>>, (axum::http::StatusCode, String)> {
    let workflows = sqlx::query_as!(
        ResearchWorkflow,
        "SELECT * FROM research_workflows ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(workflows))
}

#[instrument(skip(pool))]
pub async fn get_research_workflow(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ResearchWorkflow>, (axum::http::StatusCode, String)> {
    let workflow = sqlx::query_as!(
        ResearchWorkflow,
        "SELECT * FROM research_workflows WHERE id = $1",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            "Research workflow not found".to_string(),
        )
    })?;

    Ok(Json(workflow))
}

#[instrument(skip(pool, payload))]
pub async fn create_research_workflow(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateResearchWorkflow>,
) -> Result<Json<ResearchWorkflow>, (axum::http::StatusCode, String)> {
    let workflow = sqlx::query_as!(
        ResearchWorkflow,
        r#"
        INSERT INTO research_workflows (name, description, steps)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        payload.name,
        payload.description,
        payload.steps
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(workflow))
}

#[instrument(skip(pool, payload))]
pub async fn update_research_workflow(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateResearchWorkflow>,
) -> Result<Json<ResearchWorkflow>, (axum::http::StatusCode, String)> {
    let workflow = sqlx::query_as!(
        ResearchWorkflow,
        r#"
        UPDATE research_workflows
        SET 
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            steps = COALESCE($3, steps),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $4
        RETURNING *
        "#,
        payload.name,
        payload.description,
        payload.steps,
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            "Research workflow not found".to_string(),
        )
    })?;

    Ok(Json(workflow))
}

#[instrument(skip(pool))]
pub async fn delete_research_workflow(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, (axum::http::StatusCode, String)> {
    sqlx::query("DELETE FROM research_workflows WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(()))
}
