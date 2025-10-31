//! Defines the request body structure for creating a new style from an existing creative.
//!
//! This struct outlines the fields required when a client sends a request
//! to the create style from creative endpoint. It includes the ID of the source creative
//! and the desired name for the new style.
//! Uses Serde for deserialization and Utoipa for schema generation.

/// Request body for creating a new style from an existing creative.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateStyleFromCreativeRequest {
    /// The UUID of the creative to use as a source for the new style.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub creative_id: uuid::Uuid,
    /// The name for the new style.
    #[schema(example = "Style from Creative X")]
    pub name: std::string::String,

    /// Optional organization ID to deduct credits from (if user is acting on behalf of an organization)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = Option<String>)]
    #[serde(default)]
    pub organization_id: Option<uuid::Uuid>,
}

// No tests needed for a simple data structure definition.
