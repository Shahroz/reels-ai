use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Uuid;
use sqlx::FromRow;
use utoipa::ToSchema;

/// Represents a user-defined creative format.
#[derive(Debug, Clone, FromRow, ToSchema, Serialize, Deserialize)]
pub struct CustomCreativeFormat {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: Uuid,
    #[schema(format = "uuid", value_type=Option<String>, nullable = true, example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    #[schema(example = 1920, nullable = true)]
    pub width: Option<i32>, // Already optional in created file
    #[schema(example = 1080, nullable = true)]
    pub height: Option<i32>, // Already optional in created file
    #[schema(example = "image")] // Represents the type, e.g., 'website', 'image', 'data'
    pub creative_type: String, // Ensure this is String to match TEXT column
    #[schema(value_type = Object, example = json!({"schema": "details"}))]
    pub json_schema: Option<Value>, // Already added in created file
    #[schema(example = false)]
    pub is_public: bool,
    pub metadata: Option<Value>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub updated_at: sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>,
}
