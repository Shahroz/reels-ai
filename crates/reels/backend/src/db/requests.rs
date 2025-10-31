// --READONLY comment if creating a migration
// This file handles database operations for the 'requests' table.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, PgPool};
use tracing::instrument;


// Corresponds to the 'requests' table schema
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct RequestRecord {
    pub id: i32,
    pub url: Option<String>,
    pub content_to_style: Option<String>,
    #[schema(value_type = String, example = "2025-04-09T12:39:46Z")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[schema(value_type = String, example = "2025-04-09T12:39:46Z")]
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    #[schema(value_type = String, example = "2025-04-09T12:39:46Z")]
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub what_to_create: Option<String>,
    pub compressed_style_website_content: Option<Vec<u8>>,
    pub compressed_output_html: Option<Vec<u8>>,
    pub status: Option<String>,
    pub execution_time_ms: Option<i32>,
    #[schema(value_type = String, example = "00000000-0000-0000-0000-000000000001")]
    pub user_id: Option<Uuid>,
    pub visual_feedback: Option<bool>,
    pub credits_used: Option<i32>,
    pub favourite: bool,
}

#[derive(Debug, Clone)]
pub struct CreateRequestArgs {
    pub url: Option<String>,
    pub content_to_style: Option<String>,
    pub what_to_create: Option<String>,
    pub status: String,
    pub user_id: Option<Uuid>,
    pub visual_feedback: Option<bool>,
}

/// Creates a new request record and returns its ID.
#[instrument(skip(pool, args))]
pub async fn create_request(pool: &PgPool, args: CreateRequestArgs) -> anyhow::Result<i32> {
    
    let record = sqlx::query!(
        r#"
        INSERT INTO requests (url, content_to_style, what_to_create, status, user_id, visual_feedback, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        RETURNING id
        "#,
        args.url,
        args.content_to_style,
        args.what_to_create,
        args.status,
        args.user_id,
        args.visual_feedback
    )
    .fetch_one(pool)
    .await
    .context("Failed to insert new request into database")?;

    Ok(record.id)
}

#[derive(Debug, Clone)]
pub struct UpdateRequestArgs {
    pub compressed_style_website_content: Option<Vec<u8>>,
    pub compressed_output_html: Option<Vec<u8>>,
    pub status: String,
    pub execution_time_ms: Option<i32>,
    pub credits_used: Option<i32>,
}

/// Updates an existing request record upon completion or failure.
#[instrument(skip(pool, args))]
pub async fn update_request_completion(
    pool: &PgPool,
    request_id: i32,
    args: UpdateRequestArgs,
) -> anyhow::Result<()> {
    
    sqlx::query!(
        r#"
        UPDATE requests
        SET
            finished_at = NOW(),
            compressed_style_website_content = $1,
            compressed_output_html = $2,
            status = $3,
            execution_time_ms = $4,
            credits_used = $5
        WHERE id = $6
        "#,
        args.compressed_style_website_content,
        args.compressed_output_html,
        args.status,
        args.execution_time_ms,
        args.credits_used,
        request_id
    )
    .execute(pool)
    .await
    .context("Failed to update request status in database")?;

    Ok(())
}

/// Updates the status of a request record.
#[instrument(skip(pool))]
pub async fn update_request_status(
    pool: &PgPool,
    request_id: i32,
    status: &str,
) -> anyhow::Result<()> {
    
    sqlx::query!(
        "UPDATE requests SET status = $1 WHERE id = $2",
        status,
        request_id
    )
    .execute(pool)
    .await
    .context("Failed to update request status")?;
    Ok(())
}

// TODO: Add functions for fetching requests, marking as favourite, etc. as needed.
