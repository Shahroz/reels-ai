//! Defines the CollectionItem struct, representing an item within a collection.
//!
//! This struct corresponds to the `db_collection_items` table and stores data
//! associated with individual items in a user's collection, including custom metadata.
//! It links back to a parent Collection.

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CollectionItem {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: uuid::Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type=String)]
    pub collection_id: uuid::Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440002", format = "uuid", value_type=String)]
    pub user_id: uuid::Uuid,
    #[schema(value_type = Object, example = json!({"content": "item data"}))]
    pub data: serde_json::Value,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
