//! Response structure for listing API keys with user details.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use utoipa::ToSchema;

/// Enhanced API key metadata with user details.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiKeyWithUserDetails {
    #[schema(value_type = String)]
    pub id: Uuid,
    #[schema(value_type = String)]
    pub user_id: Uuid,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_used_at: Option<DateTime<Utc>>,
    /// Comma-separated list of allowed domains for API key usage
    pub allowed_domains: Option<String>,
    /// User email address
    pub user_email: String,
    /// Whether user's email is verified
    pub user_email_verified: bool,
    /// Whether user is an admin
    pub user_is_admin: bool,
    /// User account status
    pub user_status: String,
    /// User account creation date
    #[schema(value_type = String, format = DateTime)]
    pub user_created_at: DateTime<Utc>,
}
