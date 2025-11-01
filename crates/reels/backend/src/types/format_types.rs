//! Types for custom creative format operations

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request for creating or updating a custom creative format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomFormatRequest {
    pub name: String,
    pub description: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub json_schema: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub creative_type: String,
    pub is_public: Option<bool>,
}

