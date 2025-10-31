use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct WebflowCreative {
    pub id: Uuid,
    pub document_ids: Option<Vec<Uuid>>, // Renamed from research_ids
    pub publish_url: String,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
}
