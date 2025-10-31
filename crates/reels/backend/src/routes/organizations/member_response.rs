use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::types::Uuid;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, FromRow, Serialize, ToSchema, serde::Deserialize)]
pub struct OrganizationMemberResponse {
    // Fields from OrganizationMember
    #[schema(format = "uuid")]
    pub user_id: Uuid,
    pub role: String,
    pub status: String,
    #[schema(format = "uuid")]
    pub invited_by_user_id: Option<Uuid>,
    pub invited_at: Option<DateTime<Utc>>,
    pub joined_at: Option<DateTime<Utc>>,

    // Additional fields from User table
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "Jane Doe")]
    pub name: Option<String>, // Assuming name is optional in users table
    // Add avatar_url if you have it and want to display it
    // pub avatar_url: Option<String>,
} 