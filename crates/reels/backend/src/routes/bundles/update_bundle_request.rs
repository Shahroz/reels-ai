//! Defines the request payload for updating an existing bundle.
//!
//! All fields are optional. Adheres to `zenide.md` for Utoipa schema annotations.

// Revision History (New File)
// - 2025-05-29T15:27:46Z @AI: Initial implementation.

use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateBundleRequest {
    #[schema(example = "My Updated Marketing Bundle")]
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[schema(example = "Updated description for Q3 marketing campaigns.")]
    #[validate(length(max = 1000))]
    pub description: Option<Option<String>>, // Outer Option for presence, inner for nullable
    #[schema(example = "c3d4e5f6-a7b8-9012-3456-7890abcdef12", format = "uuid", value_type = Option<String>)]
    pub style_id: Option<Uuid>,
    #[schema(value_type = Option<Vec<String>>, example = json!(["d4e5f6a7-b8c9-0123-4567-890abcdef123"]))]
    pub document_ids: Option<Vec<Uuid>>,
    #[schema(value_type = Option<Vec<String>>, example = json!(["e5f6a7b8-c9d0-1234-5678-90abcdef1234"]))]
    pub asset_ids: Option<Vec<Uuid>>,
    #[schema(value_type = Option<Vec<String>>, example = json!(["f6a7b8c9-d0e1-2345-6789-0abcdef12345"]))]
    pub format_ids: Option<Vec<Uuid>>,
}