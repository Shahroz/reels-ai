/// Represents a style in an API response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct StyleResponse {
    #[serde(flatten)]
    #[schema(inline)]
    pub style: crate::db::styles::Style,
    #[schema(example = "user@example.com", nullable = true)]
    pub creator_email: std::option::Option<std::string::String>,
    /// The current authenticated user's access level to this style (e.g., owner, editor, viewer).
    #[schema(example = "editor", nullable = true)]
    pub current_user_access_level: std::option::Option<std::string::String>,
}

/// Represents a style returned by the API, including associated metadata and favorite status.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct StyleResponseWithFavorite {
    #[serde(flatten)]
    #[schema(inline)]
    pub style: crate::db::styles::Style,
    #[schema(example = "user@example.com")]
    pub creator_email: std::option::Option<std::string::String>,
    #[schema(example = "owner")]
    pub current_user_access_level: std::option::Option<std::string::String>,
    #[schema(example = false)]
    pub is_favorite: bool,
} 