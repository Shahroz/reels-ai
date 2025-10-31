//! Response types for collections endpoints that include permission details.
//!
//! These types are used when returning collection data that includes
//! the current user's access level for permission-aware frontends.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use utoipa::ToSchema;

/// Collection response that includes the current user's access level
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CollectionWithPermissions {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub user_id: Uuid,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
    /// Current user's access level to this collection ('owner', 'editor', 'viewer', or null)
    #[schema(example = "owner")]
    pub current_user_access_level: Option<String>,
}

/// Response for listing collections with permission details
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListCollectionsWithPermissionsResponse {
    pub items: Vec<CollectionWithPermissions>,
    pub total_count: i64,
}
