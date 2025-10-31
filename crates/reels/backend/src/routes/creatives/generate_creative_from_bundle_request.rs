//! Defines the request payload for generating a creative from a bundle.
//!
//! Adheres to one-item-per-file guideline and Utoipa schema annotations.

use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct GenerateCreativeFromBundleRequest {
    /// Required name for the creative.
    #[schema(example = "My Creative", value_type = String)]
    pub name: std::string::String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440005", format = "uuid", value_type = String)]
    pub collection_id: Uuid,

    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type = String)]
    pub bundle_id: Uuid,

    #[schema(value_type = Option<Vec<String>>, example = json!(["d4e5f6a7-b8c9-0123-4567-890abcdef123"]))]
    pub document_ids: Option<Vec<Uuid>>, // Optional: If provided, these specific documents are used. If None/empty, bundle.document_ids are used.

    /// Optional organization ID to deduct credits from (if user is acting on behalf of an organization)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = Option<String>)]
    #[serde(default)]
    pub organization_id: Option<Uuid>,
}