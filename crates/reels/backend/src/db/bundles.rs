//! Defines the `Bundle` struct, which represents the schema for the `bundles` database table.
//!
//! A Bundle is a collection of resources (style guide, documents, assets, formats)
//! that can be applied to tasks like research and creative generation.
//! This file solely defines the `Bundle` struct, representing the `bundles` table schema.
//! Query functions are located in `crate::queries::bundles`.

//! Revision History
//! - 2025-05-29T18:15:36Z @AI: Refactored: Moved query functions to crates/narrativ/backend/src/queries/bundles/. Retained Bundle struct definition.
//! - 2025-05-29T15:27:46Z @AI: Initial implementation of Bundle struct and CRUD functions. (Preserved original history)

/// Represents a bundle of resources.
#[derive(std::fmt::Debug, std::clone::Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Bundle {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type = String)]
    pub id: sqlx::types::Uuid,
    #[schema(example = "b2c3d4e5-f6a7-8901-2345-67890abcdef1", format = "uuid", value_type = String)]
    pub user_id: sqlx::types::Uuid,
    #[schema(example = "My Awesome Bundle")]
    pub name: String,
    #[schema(example = "A description of what this bundle contains.")]
    pub description: Option<String>,
    #[schema(example = "c3d4e5f6-a7b8-9012-3456-7890abcdef12", format = "uuid", value_type = String)]
    pub style_id: sqlx::types::Uuid,
    #[schema(value_type = Vec<String>, example = json!(["d4e5f6a7-b8c9-0123-4567-890abcdef123"]))]
    pub document_ids: Vec<sqlx::types::Uuid>,
    #[schema(value_type = Vec<String>, example = json!(["e5f6a7b8-c9d0-1234-5678-90abcdef1234"]))]
    pub asset_ids: Vec<sqlx::types::Uuid>,
    #[schema(value_type = Vec<String>, example = json!(["f6a7b8c9-d0e1-2345-6789-0abcdef12345"]))]
    pub format_ids: Vec<sqlx::types::Uuid>,
    #[schema(value_type = String, format = "date-time", example = "2024-05-29T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-05-29T12:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}