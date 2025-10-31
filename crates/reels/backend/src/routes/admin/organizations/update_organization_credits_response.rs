//! Response structure for organization credit update operations.
//!
//! This struct provides feedback after successfully updating an organization's credits
//! via the admin endpoint. It includes the organization ID and the new credit balance
//! for confirmation and display purposes in admin interfaces.

/// Response payload for organization credit updates
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UpdateOrganizationCreditsResponse {
    /// The organization ID whose credits were updated
    #[schema(example = "660e8400-e29b-41d4-a716-446655440111", format = "uuid", value_type = String)]
    pub organization_id: uuid::Uuid,
    
    /// The new credit balance
    #[schema(example = "5000.00", value_type = String)]
    pub credits_remaining: bigdecimal::BigDecimal,
    
    /// Success message
    #[schema(example = "Organization credits updated successfully")]
    pub message: String,
}

