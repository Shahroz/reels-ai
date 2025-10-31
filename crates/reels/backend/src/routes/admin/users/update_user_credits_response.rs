//! Response structure for user credit update operations.
//!
//! This struct provides feedback after successfully updating a user's credits via the
//! admin endpoint. It includes the user ID, organization ID, and the new credit balance
//! for confirmation and display purposes in admin interfaces.

/// Response payload for user credit updates
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UpdateUserCreditsResponse {
    /// The user ID whose credits were updated
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub user_id: uuid::Uuid,
    
    /// The personal organization ID where credits are stored
    #[schema(example = "660e8400-e29b-41d4-a716-446655440111", format = "uuid", value_type = String)]
    pub organization_id: uuid::Uuid,
    
    /// The new credit balance
    #[schema(example = "1000.00", value_type = String)]
    pub credits_remaining: bigdecimal::BigDecimal,
    
    /// Success message
    #[schema(example = "User credits updated successfully")]
    pub message: String,
}

