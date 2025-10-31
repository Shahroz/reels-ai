//! Defines the VocalTour struct, representing a vocal tour entity in the database.
//!
//! This struct maps to the `vocal_tours` table in the database and tracks the relationship
//! between vocal tour documents and their generated assets. Each vocal tour is associated
//! with exactly one document and multiple assets that were created during the vocal tour
//! processing workflow. This enables linking vocal tour outputs to property listings.

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct VocalTour {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type = String)]
    pub user_id: uuid::Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440002", format = "uuid", value_type = String)]
    pub document_id: uuid::Uuid,
    #[schema(example = r#"["550e8400-e29b-41d4-a716-446655440003", "550e8400-e29b-41d4-a716-446655440004"]"#)]
    pub asset_ids: std::vec::Vec<uuid::Uuid>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T12:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
} 