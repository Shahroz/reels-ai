//! Favorite with entity data model for API responses.
use crate::db::favorites::{FavoriteEntityType};
use crate::routes::user_favorites::entity_relations::EntityData;
use serde::{Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Favorite with entity data for API responses
#[derive(Serialize, Debug, ToSchema)]
pub struct FavoriteWithEntity {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    #[schema(value_type = String, format = "uuid")]
    pub user_id: Uuid,
    #[schema(value_type = String, format = "uuid")]
    pub entity_id: Uuid,
    pub entity_type: FavoriteEntityType,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
    /// Entity data - null if entity doesn't exist or user doesn't have access
    #[schema(nullable = true)]
    pub entity_data: Option<EntityData>,
}