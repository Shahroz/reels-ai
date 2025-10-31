// backend/src/db/documents.rs
// Data models for `research` table
/// Represents a document, potentially generated from research or other sources.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema, PartialEq)]
pub struct Document {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: sqlx::types::Uuid,
    #[schema(format = "uuid", value_type=Option<String>, nullable = true, example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: Option<sqlx::types::Uuid>,
    #[schema(example = "Market Analysis for AI Startups")]
    pub title: String,
    #[schema(example = "Detailed analysis of market trends...")]
    pub content: String,
    #[schema(example = json!(["https://example.com/source1", "https://anothersource.org"]))]
    pub sources: Vec<String>,
    #[schema(example = "Completed")]
    pub status: String,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[schema(example = false)]
    pub is_public: bool,
    pub is_task: bool,
    pub include_research: Option<crate::db::document_research_usage::DocumentResearchUsage>,
    #[schema(format = "uuid", value_type = Option<String>, nullable = true, example = "550e8400-e29b-41d4-a716-446655440001")]
    pub collection_id: Option<sqlx::types::Uuid>,
}
