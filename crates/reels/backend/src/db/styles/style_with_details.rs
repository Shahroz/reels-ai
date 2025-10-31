//! StyleWithDetails database model for styles query results.
//!
//! This struct represents a style with additional details including
//! creator information, access level, and favorite status for queries.
//! Used specifically for enriched style listing operations.

#[derive(sqlx::FromRow, Debug, serde::Serialize, serde::Deserialize)]
pub struct StyleWithDetails {
    pub id: uuid::Uuid,
    pub user_id: std::option::Option<uuid::Uuid>,
    pub name: std::string::String,
    pub html_url: std::string::String,
    pub screenshot_url: std::string::String,
    pub is_public: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub creator_email: std::option::Option<std::string::String>,
    pub current_user_access_level: std::option::Option<std::string::String>,
    pub is_favorite: std::option::Option<bool>,
}
