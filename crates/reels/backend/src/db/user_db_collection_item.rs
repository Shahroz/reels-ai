//! Defines the UserDbCollectionItem struct, representing an item within a custom user collection.
//!
//! This struct maps to the `user_db_collection_items` table in the database.
//! It stores the actual data for an item, which must conform to the JSON schema
//! defined in the parent `UserDbCollection`. Each item belongs to a specific collection.

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct UserDbCollectionItem {
    #[schema(format = "uuid", value_type=String)]
    pub id: uuid::Uuid,
    #[schema(format = "uuid", value_type=String)]
    pub user_db_collection_id: uuid::Uuid,
    pub item_data: serde_json::Value,
    #[schema(format = "date-time", value_type=String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(format = "date-time", value_type=String)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// Note: No functions (CRUD operations) are defined in this file as per the initial request.
// Those would typically be added here or in a related service module.
// Adhering to "One Logical Item Per File" - this file defines the UserDbCollectionItem struct.
// No `use` statements are included, as per rust_guidelines.
// Fully qualified paths are expected for types like uuid::Uuid, serde_json::Value, chrono::DateTime, etc.
