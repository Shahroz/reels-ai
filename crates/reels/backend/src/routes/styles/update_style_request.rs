//! Defines the request body structure for updating an existing style.
//!
//! This struct specifies the data needed to modify a style resource.
//! It requires the updated name, the full HTML content for the style,
//! and optionally whether the style should be public.
//! Used in the PUT request handler for styles.
//! Includes Serde and Utoipa integration.

/// Request body for updating an existing style.
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct UpdateStyleRequest {
    #[schema(example = "Updated Style Name")]
    pub name: std::string::String,
    #[schema(example = "<style>p { color: blue; }</style>")]
    /// Optional URL to fetch style from, overrides html_content if provided
    #[schema(example = "https://example.com")]
    pub source_url: Option<std::string::String>,
    pub html_content: std::string::String,
    /// Whether the style should be public (accessible to all users)
    #[schema(example = false)]
    pub is_public: Option<bool>,
}
