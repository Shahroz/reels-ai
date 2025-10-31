//! Defines the UserDbCollection struct, representing a custom user-defined collection.
//!
//! This struct maps to the `user_db_collections` table in the database.
//! It stores metadata for a collection, including its name, description,
//! and a JSON schema defining the structure of items within that collection.
//! Each collection is associated with a specific user.

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct UserDbCollection {
    #[schema(format = "uuid", value_type=String)]
    pub id: uuid::Uuid,
    #[schema(format = "uuid", value_type=String)]
    pub user_id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub schema_definition: serde_json::Value,
    #[schema(format = "uuid", value_type=String, nullable = true)]
    pub source_predefined_collection_id: Option<uuid::Uuid>,
    pub ui_component_definition: serde_json::Value,
    #[schema(format = "date-time", value_type=String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(format = "date-time", value_type=String)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// Note: No functions (CRUD operations) are defined in this file as per the initial request.
// Those would typically be added here or in a related service module.
// Adhering to "One Logical Item Per File" - this file defines the UserDbCollection struct.
// No `use` statements are included, as per rust_guidelines.
// Fully qualified paths are expected for types like uuid::Uuid, serde_json::Value, chrono::DateTime, etc.
