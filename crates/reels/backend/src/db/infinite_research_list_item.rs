//! Represents an infinite research task for list views, including last execution details.
//!
//! This struct joins the core infinite research data with key information
//! from its most recent execution, such as status and start time.
//! It is designed for use in API responses where a summary is needed.

#[derive(sqlx::FromRow, serde::Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct InfiniteResearchListItem {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub name: std::string::String,
    pub prompt: std::string::String,
    pub cron_schedule: std::string::String,
    pub is_enabled: bool,
    pub scheduler_job_name: std::option::Option<std::string::String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = Option<String>, format = "uuid", nullable = true)]
    pub last_execution_id: std::option::Option<uuid::Uuid>,
    #[schema(value_type = Option<String>, format = "date-time", nullable = true)]
    pub last_execution_started_at: std::option::Option<chrono::DateTime<chrono::Utc>>,
    #[schema(nullable = true)]
    pub last_execution_status: std::option::Option<std::string::String>,
}