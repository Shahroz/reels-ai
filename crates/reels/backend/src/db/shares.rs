use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "object_share_entity_type", rename_all = "snake_case")]
pub enum EntityType {
    User,
    Organization,
}

impl FromStr for EntityType {
    type Err = ();

    fn from_str(input: &str) -> Result<EntityType, Self::Err> {
        match input.to_lowercase().as_str() {
            "user" => Ok(EntityType::User),
            "organization" => Ok(EntityType::Organization),
            _ => Err(()),
        }
    }
}

impl ToString for AccessLevel {
    fn to_string(&self) -> String {
        match self {
            AccessLevel::Viewer => "viewer".to_string(),
            AccessLevel::Editor => "editor".to_string(),
        }
    }
}



#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "object_share_access_level", rename_all = "snake_case")]
pub enum AccessLevel {
    Viewer,
    Editor,
}

impl FromStr for AccessLevel {
    type Err = ();

    fn from_str(input: &str) -> Result<AccessLevel, Self::Err> {
        match input.to_lowercase().as_str() {
            "viewer" => Ok(AccessLevel::Viewer),
            "editor" => Ok(AccessLevel::Editor),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct ObjectShare {
    pub id: Uuid,
    pub object_id: Uuid,
    pub object_type: String,
    pub entity_id: Uuid,
    pub entity_type: EntityType,
    pub access_level: AccessLevel,
    pub entity_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
} 
