use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RetouchParams {
    /// Prompt used for retouch/enhancement
    pub retouch_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum DerivationParams {
    Retouch(RetouchParams),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StudioGraphNode {
    pub asset_id: sqlx::types::Uuid,
    pub url: String,
    pub name: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>, 
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StudioGraphEdge {
    pub source_asset_id: sqlx::types::Uuid,
    pub derived_asset_id: sqlx::types::Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StudioGraph {
    pub nodes: Vec<StudioGraphNode>,
    pub edges: Vec<StudioGraphEdge>,
    pub root_asset_id: sqlx::types::Uuid,
    /// Journey ID in database (if journey has been created)
    pub journey_id: Option<sqlx::types::Uuid>,
}


