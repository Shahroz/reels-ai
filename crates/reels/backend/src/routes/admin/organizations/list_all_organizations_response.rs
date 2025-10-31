//! Defines the response structure for the list all organizations endpoint.
//!
//! This struct wraps the paginated list of enriched organizations with metadata.
//! Used by admin interfaces to display all organizations with owner and member details.

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct EnrichedOrganizationDto {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type = String)]
    pub id: uuid::Uuid,

    #[schema(example = "Acme Corporation")]
    pub name: String,

    #[schema(example = "b2c3d4e5-f6a7-8901-2345-67890abcdef1", format = "uuid", value_type = String)]
    pub owner_user_id: uuid::Uuid,

    #[schema(example = "owner@acme.com")]
    pub owner_email: String,

    #[schema(example = 5)]
    pub member_count: i64,

    #[schema(value_type = String, format = "date-time", example = "2024-05-05T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[schema(value_type = String, format = "date-time", example = "2024-05-05T12:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,

    #[schema(example = "5000.00", value_type = Option<String>)]
    pub credits_remaining: Option<bigdecimal::BigDecimal>,
}

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ListAllOrganizationsResponse {
    /// A list of enriched organizations with owner and member details.
    pub items: Vec<EnrichedOrganizationDto>,

    /// The total number of organizations matching the query filters.
    pub total_count: i64,

    /// The current page number.
    pub page: i64,

    /// The number of items per page.
    pub limit: i64,
}
