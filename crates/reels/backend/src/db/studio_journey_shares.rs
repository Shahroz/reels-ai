//! Defines the database model for a Studio Journey Share.
//!
//! This struct corresponds to the `studio_journey_shares` table and holds all the
//! data related to a public share link for a user's creative journey.
//! It includes the share token, activation status, and timestamps.

/// Represents a public share link for a Studio Journey.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct StudioJourneyShare {
    /// The unique identifier for the share record.
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type=String)]
    pub id: uuid::Uuid,

    /// The ID of the journey being shared.
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type=String)]
    pub journey_id: uuid::Uuid,

    /// The secret token used for public access.
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type=String)]
    pub share_token: uuid::Uuid,

    /// Whether the share link is currently active and accessible.
    pub is_active: bool,

    /// The timestamp when the share was created.
    #[schema(value_type = String, example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// The timestamp when the share was last updated.
    #[schema(value_type = String, example = "2024-04-21T10:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}