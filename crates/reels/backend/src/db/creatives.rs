//! Data models for `creatives` table.
//!
//! Represents an HTML-based creative output.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize}; // Added Deserialize
use sqlx::types::Uuid;
use sqlx::FromRow;
use utoipa::ToSchema;

/// Represents an HTML-based creative output.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)] // Added Deserialize
pub struct Creative {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: Uuid,
    #[schema(example = "My Creative", value_type = String)]
    pub name: String,
    #[schema(example = "coll_550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=Option<String>)]
    pub collection_id: Option<Uuid>,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    // Added
    pub creative_format_id: Uuid, // Added, obligatory
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=Option<String>)]
    pub style_id: Option<Uuid>,
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440003"]), format = "uuid", value_type=Option<Vec<String>>)]
    // Changed to list
    pub document_ids: Option<Vec<Uuid>>, // Renamed from research_ids
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440001", "550e8400-e29b-41d4-a716-446655440002"]), value_type=Option<Vec<String>>)]
    // Added
    pub asset_ids: Option<Vec<Uuid>>, // Added, optional list
    #[schema(example = "https://storage.googleapis.com/your-bucket/creatives/{id}/creative.html", value_type = String, format = "uri")]
    pub html_url: String,
    #[schema(example = "https://storage.googleapis.com/your-bucket/creatives/{id}/creative.html", value_type = String, format = "uri")]
    pub draft_url: Option<String>,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440006", format = "uuid", value_type=Option<String>)]
    pub bundle_id: Option<Uuid>, // Added bundle_id
    #[schema(example = "https://storage.googleapis.com/your-bucket/creatives/{id}/screenshot.png", value_type = String, format = "uri")]
    pub screenshot_url: String,
    #[schema(example = false)] // Added
    pub is_published: bool, // Added, default false handled by DB
    #[schema(example = "https://example.com/published/creative")] // Added
    pub publish_url: Option<String>, // Added, optional
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
}