//! Represents a single infinite research task as stored in the database.
//!
//! This struct defines the shape of an infinite research record, including its
//! configuration, status, and associated metadata. It is used across various
//! query and route handlers.

/// Represents a single infinite research task as stored in the database.
#[derive(sqlx::FromRow, serde::Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct InfiniteResearch {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub name: std::string::String,
    pub prompt: std::string::String,
    pub cron_schedule: std::string::String,
    pub is_enabled: bool,
    pub scheduler_job_name: std::option::Option<std::string::String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
