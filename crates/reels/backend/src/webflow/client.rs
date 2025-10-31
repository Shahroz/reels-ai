use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebflowError {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Serde JSON error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateItemPayload {
    field_data: HashMap<String, Value>,
    is_draft: bool,
    is_archived: bool,
}

// Basic structure for the response
#[derive(Deserialize, Debug, PartialEq)]
pub struct ItemResponse {
    pub id: String,
    #[serde(rename = "fieldData")]
    pub field_data: HashMap<String, Value>,
    #[serde(rename = "lastPublished")]
    pub last_published: Option<String>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
    #[serde(rename = "createdOn")]
    pub created_on: Option<String>,
    #[serde(rename = "isArchived")]
    pub is_archived: bool,
    #[serde(rename = "isDraft")]
    pub is_draft: bool,
}

// Structure for collection items response
#[derive(Deserialize, Debug)]
pub struct CollectionItemsResponse {
}

#[derive(Deserialize, Debug)]
pub struct PaginationInfo {
}
