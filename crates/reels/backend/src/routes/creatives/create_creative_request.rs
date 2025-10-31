//! Defines the request body structure for creating a new creative.
//!
//! This structure captures the necessary fields required to initiate
//! the creation of a creative entity via the API endpoint.
//! It includes optional and required identifiers for associated resources.
//! Fields are public to allow direct access.

// Note: uuid::Uuid might be in prelude or require FQN depending on crate setup. Assuming FQN for safety.
// Note: Use std::string::String etc. for FQN where needed, but not inside utoipa annotations per zenide.md.
// Adhering to no 'use' statements guideline.

/// Request payload for creating a new creative.
#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)] // Keep derives as they are, assuming they work without explicit 'use'.
pub struct CreateCreativeRequest {
    /// Required name for the creative.
    #[schema(example = "My Creative", value_type = String)]
    pub name: std::string::String,
    /// Optional ID of the collection this creative belongs to.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440005", format = "uuid", value_type = Option<String>)]
    pub collection_id: Option<uuid::Uuid>, // Assuming uuid::Uuid is correct FQN or in prelude
    /// Required ID of the creative format to be used.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub creative_format_id: uuid::Uuid, // Assuming uuid::Uuid is correct FQN or in prelude
    /// Optional ID of the style to apply.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440003", format = "uuid", value_type = Option<String>)]
    pub style_id: Option<uuid::Uuid>, // Assuming uuid::Uuid is correct FQN or in prelude
    /// Optional list of document item IDs associated with this creative.
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440001", "550e8400-e29b-41d4-a716-446655440002"]), value_type = Option<Vec<String>>)]
    pub document_ids: Option<std::vec::Vec<uuid::Uuid>>, // Assuming uuid::Uuid is correct FQN or in prelude
    /// Optional list of asset IDs to be included in this creative.
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440001", "550e8400-e29b-41d4-a716-446655440002"]), value_type = Option<Vec<String>>)]
    pub asset_ids: Option<std::vec::Vec<uuid::Uuid>>, // Assuming uuid::Uuid is correct FQN or in prelude
    /// URL of the HTML file in GCS for this creative.
    #[schema(example = "https://storage.googleapis.com/your-bucket/creatives/{id}/creative.html", value_type = String, format = "uri")]
    pub html_url: std::string::String,
    /// URL of the screenshot image in GCS for this creative.
    #[schema(example = "https://storage.googleapis.com/your-bucket/creatives/{id}/screenshot.png", value_type = String, format = "uri")]
    pub screenshot_url: std::string::String,
}

// No tests needed for a simple data structure definition.
