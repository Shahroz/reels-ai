// backend/src/db/assets.rs
// Data models for `assets` table

// use chrono::{DateTime, Utc}; // Removed unused imports
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::FromRow;
use utoipa::ToSchema;

/// Represents a user-uploaded asset stored in Google Cloud Storage.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct Asset {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: Uuid,
    #[schema(format = "uuid", value_type=Option<String>, nullable = true, example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: Option<Uuid>,
    pub name: String,
    pub r#type: String,
    pub gcs_object_name: String,
    pub url: String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type=String, nullable = true)]
    pub collection_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T12:00:00Z")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[schema(example = false)]
    pub is_public: bool,
}

/// Represents an asset with provenance information (whether it's enhanced/derived or original).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AssetWithProvenance {
    #[serde(flatten)]
    pub asset: Asset,
    #[schema(example = false)]
    pub is_enhanced: bool,
}
