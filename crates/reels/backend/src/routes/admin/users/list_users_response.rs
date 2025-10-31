//! Defines the response structure for the list users endpoint.

use serde::Serialize;
use utoipa::ToSchema;

/// Enriched user data with credits information
#[derive(Debug, Serialize, serde::Deserialize, ToSchema)]
pub struct EnrichedUserDto {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    pub email: String,
    pub status: String,
    pub is_admin: bool,
    pub feature_flags: Vec<String>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(example = "1000.00", value_type = Option<String>)]
    pub credits_remaining: Option<bigdecimal::BigDecimal>,
    #[schema(example = false)]
    pub is_unlimited: bool,
}

/// Response payload for listing users.
#[derive(Serialize, ToSchema)]
pub struct ListUsersResponse {
    /// A list of users with credit information.
    pub items: Vec<EnrichedUserDto>,
    /// The total number of users matching the query.
    pub total_count: i64,
}