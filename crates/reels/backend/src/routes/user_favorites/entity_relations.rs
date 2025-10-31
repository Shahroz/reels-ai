//! Entity relation models for user favorites API.
use crate::db::{styles::Style, creatives::Creative, documents::Document};
use serde::{Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Style entity data for favorites response
#[derive(Serialize, Debug, ToSchema)]
pub struct StyleEntity {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    pub name: String,
    pub html_url: String,
    pub screenshot_url: String,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

/// Creative entity data for favorites response
#[derive(Serialize, Debug, ToSchema)]
pub struct CreativeEntity {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    #[schema(value_type = String, format = "uuid", nullable = true)]
    pub collection_id: Option<Uuid>,
    #[schema(value_type = String, format = "uuid")]
    pub creative_format_id: Uuid,
    #[schema(value_type = String, format = "uuid", nullable = true)]
    pub style_id: Option<Uuid>,
    pub screenshot_url: String,
    pub is_published: bool,
    #[schema(nullable = true)]
    pub html_url: Option<String>,
    #[schema(nullable = true)]
    pub draft_url: Option<String>,
    #[schema(nullable = true)]
    pub publish_url: Option<String>,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
    #[schema(nullable = true)]
    pub style_name: Option<String>,
    #[schema(nullable = true)]
    pub creative_format_name: Option<String>,
    #[schema(nullable = true)]
    pub collection_name: Option<String>,
}

/// Document entity data for favorites response
#[derive(Serialize, Debug, ToSchema)]
pub struct DocumentEntity {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub sources: Vec<String>,
    pub status: String,
    pub is_public: bool,
    pub is_task: bool,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

/// Union type for entity data in favorites response
#[derive(Serialize, Debug, ToSchema)]
#[serde(untagged)]
pub enum EntityData {
    Style(Style),
    Creative(Creative),
    Document(Document),
}