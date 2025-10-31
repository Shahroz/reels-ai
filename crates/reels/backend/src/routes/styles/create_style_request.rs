//! Defines the request body structure for creating a new style.
//!
//! This struct outlines the fields required when a client sends a request
//! to the create style endpoint. It includes the style name, optional
//! source URL or direct HTML content, and whether the style should be public.
//! Validation occurs in the handler. Uses Serde for deserialization and Utoipa for schema generation.

/// Request body for creating a new style.
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct CreateStyleRequest {
    #[schema(example = "My Custom Style")]
    pub name: std::string::String,
    /// URL to extract style from
    #[schema(example = "https://example.com")]
    pub source_url: Option<std::string::String>,
    /// HTML content for manual style creation
    #[schema(value_type = Option<String>, example = "<style>body { background: red; }</style>")]
    pub html_content: Option<std::string::String>,
    /// Whether the style should be public (accessible to all users)
    #[schema(example = false)]
    pub is_public: Option<bool>,

    /// Optional organization ID to deduct credits from (if user is acting on behalf of an organization)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = Option<String>)]
    #[serde(default)]
    pub organization_id: Option<uuid::Uuid>,
}
