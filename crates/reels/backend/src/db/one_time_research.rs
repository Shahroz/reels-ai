//! Represents a single one-time research task as stored in the database.
//!
//! This struct defines the shape of a one-time research record, including its
//! prompt, status, and associated metadata. It is used across various
//! query and route handlers.

/// Represents a single one-time research task as stored in the database.
#[derive(sqlx::FromRow, serde::Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct OneTimeResearch {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub prompt: std::string::String,
    pub status: std::string::String,
    pub cloud_task_name: std::option::Option<std::string::String>,
    pub output_log_url: std::option::Option<std::string::String>,
    pub error_message: std::option::Option<std::string::String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub started_at: std::option::Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: std::option::Option<chrono::DateTime<chrono::Utc>>,
    #[schema(value_type = Option<Object>)] // For utoipa
    pub progress_log: serde_json::Value,
}