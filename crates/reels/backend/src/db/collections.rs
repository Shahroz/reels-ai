// backend/src/db/collections.rs
// Data models for `collections` table

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Uuid;
use sqlx::FromRow;
use utoipa::ToSchema;

/// Represents a grouping of creatives.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct Collection {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub user_id: Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=Option<String>)]
    pub organization_id: Option<Uuid>,
    pub name: String,
    pub metadata: Option<Value>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
}
