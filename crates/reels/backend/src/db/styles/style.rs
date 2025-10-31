//! Style database model.
//!
//! Represents a style definition persisted in the database, stored in GCS.
//! Contains the core Style struct with all standard database fields.

/// Represents a style definition persisted in the database.
/// Represents a style definition persisted in the database, stored in GCS.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Style {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: uuid::Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=Option<String>)]
    pub user_id: std::option::Option<uuid::Uuid>,
    #[schema(example = "My Custom Style")]
    pub name: std::string::String,
    #[schema(example = "https://storage.googleapis.com/my-bucket/styles/550e8400-e29b-41d4-a716-446655440000/style.html", format = "uri")]
    /// GCS URL for the stored style HTML
    pub html_url: std::string::String,
    #[schema(example = "https://storage.googleapis.com/my-bucket/styles/550e8400-e29b-41d4-a716-446655440000/screenshot.png", format = "uri")]
    /// GCS URL for the style screenshot image
    pub screenshot_url: std::string::String,
    #[schema(example = false)]
    pub is_public: bool,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
