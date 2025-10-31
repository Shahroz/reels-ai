//! Defines the FileObject struct, representing a file or folder-like entity.
//!
//! This struct corresponds to the `db_file_objects` table and can represent various
//! types of items, potentially linked to a CollectionItem or organized hierarchically
//! using parent_ids. It stores essential metadata for referencing stored files.

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct FileObject {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: uuid::Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type=String)]
    pub user_id: uuid::Uuid,
    #[schema(example = "document")]
    pub item_type: String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440002", format = "uuid", value_type=Option<String>)]
    pub collection_item_id: Option<uuid::Uuid>,
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440003"]), value_type=Vec<String>)]
    pub parent_ids: Vec<uuid::Uuid>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
