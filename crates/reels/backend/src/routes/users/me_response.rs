//! Response type for the GET /api/users/me endpoint.
//!
//! This response includes the user's basic information as well as credit information
//! for all organizations they belong to. This allows clients to display both personal
//! and organizational credit balances in a single request.

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::users::PublicUser;

/// Organization credit information for the current user
///
/// Includes the organization's details and remaining credit balance.
/// Used in the /api/users/me response to show which organizations
/// the user can spend credits for.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationCreditInfo {
    /// Organization ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub organization_id: Uuid,
    
    /// Organization name
    #[schema(example = "Acme Corp")]
    pub organization_name: String,
    
    /// Remaining credits for this organization
    #[schema(example = "5000.50", value_type = String)]
    pub credits_remaining: BigDecimal,
    
    /// User's role in this organization (e.g., "owner", "member")
    #[schema(example = "owner")]
    pub user_role: String,
    
    /// Whether this is the user's personal organization
    #[schema(example = "true")]
    pub is_personal: bool,
}

/// Enhanced response for GET /api/users/me
///
/// Includes the user's basic information (PublicUser) plus credit information
/// for all organizations they belong to.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MeResponse {
    /// User's public information
    #[serde(flatten)]
    pub user: PublicUser,
    
    /// Credit information for organizations the user belongs to
    ///
    /// If the user is not a member of any organizations, this will be an empty array.
    /// Organizations without credit allocations will show 0 credits_remaining.
    pub organizations: Vec<OrganizationCreditInfo>,
    
    /// Whether this user has unlimited access (old user exempt from credit checks)
    ///
    /// True for grandfathered users with unlimited access.
    /// These users don't consume credits and shouldn't see credit UI elements.
    #[schema(example = "false")]
    pub is_unlimited: bool,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // Verifies the module compiles correctly
        assert!(true);
    }
}

