//! Response types for assets endpoints that include collection details.
//!
//! These types are used when returning asset data that may include
//! associated collection information.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use utoipa::ToSchema;

use crate::db::collections::Collection;

/// Asset response that includes collection details when available
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AssetWithCollection {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: Uuid,
    #[schema(format = "uuid", value_type=Option<String>, nullable = true, example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: Option<Uuid>,
    pub name: String,
    pub r#type: String,
    pub gcs_object_name: String,
    pub url: String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub collection_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
    #[schema(example = false)]
    pub is_public: bool,
    /// Current user's access level to this asset ('owner', 'editor', 'viewer', or null)
    #[schema(example = "owner")]
    pub current_user_access_level: Option<String>,
    /// Collection details if the asset belongs to a collection
    pub collection: Option<Collection>,
}

/// Response for listing assets with collection details
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListAssetsWithCollectionResponse {
    pub items: Vec<AssetWithCollection>,
    pub total_count: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
} 