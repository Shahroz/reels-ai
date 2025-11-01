//! Response types for queries and handlers

use serde::{Deserialize, Serialize};
// sqlx removed - no database interaction
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Collection with permission information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionWithPermissions {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub current_user_access_level: Option<String>,
}

/// Organization member response with user details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationMemberResponse {
    pub user_id: Uuid,
    pub role: String,
    pub status: String,
    pub invited_by_user_id: Option<Uuid>,
    pub invited_at: Option<DateTime<Utc>>,
    pub joined_at: Option<DateTime<Utc>>,
    pub email: String,
    pub name: Option<String>, // NULL from query, users table doesn't have name
}

/// Style response with access details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleResponse {
    // pub style: crate::db::styles::Style, // db module deleted
    pub creator_email: Option<String>,
    pub current_user_access_level: Option<String>,
}

/// API key with user details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyWithUserDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub allowed_domains: Option<Vec<String>>,
    pub user_email: String,
    pub user_email_verified: bool,
    pub user_is_admin: bool,
    pub user_status: String,
    pub user_created_at: DateTime<Utc>,
}

