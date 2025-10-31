//! Defines the PredefinedCollection struct, representing a template collection available to all users.
//!
//! This struct maps to the `predefined_collections` table in the database.
//! It stores template collections that users can copy to create their own collections.
//! Each predefined collection includes a schema definition and UI component definition.

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct PredefinedCollection {
    #[schema(format = "uuid", value_type=String)]
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub schema_definition: serde_json::Value,
    pub ui_component_definition: serde_json::Value,
    #[schema(format = "date-time", value_type=String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(format = "date-time", value_type=String)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// Note: No functions (CRUD operations) are defined in this file as per the initial request.
// Those would typically be added here or in a related service module.
// Adhering to "One Logical Item Per File" - this file defines the PredefinedCollection struct.
// No `use` statements are included, as per rust_guidelines.
// Fully qualified paths are expected for types like uuid::Uuid, serde_json::Value, chrono::DateTime, etc. 