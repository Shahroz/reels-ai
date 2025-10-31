use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "favorite_entity_type", rename_all = "snake_case")]
pub enum FavoriteEntityType {
    Style,
    Creative,
    Document,
    Prompt,
}

impl FromStr for FavoriteEntityType {
    type Err = ();

    fn from_str(input: &str) -> Result<FavoriteEntityType, Self::Err> {
        match input.to_lowercase().as_str() {
            "style" => Ok(FavoriteEntityType::Style),
            "creative" => Ok(FavoriteEntityType::Creative),
            "document" => Ok(FavoriteEntityType::Document),
            "prompt" => Ok(FavoriteEntityType::Prompt),
            _ => Err(()),
        }
    }
}

impl ToString for FavoriteEntityType {
    fn to_string(&self) -> String {
        match self {
            FavoriteEntityType::Style => "style".to_string(),
            FavoriteEntityType::Creative => "creative".to_string(),
            FavoriteEntityType::Document => "document".to_string(),
            FavoriteEntityType::Prompt => "prompt".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct UserFavorite {
    pub id: Uuid,
    pub user_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: FavoriteEntityType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
} 