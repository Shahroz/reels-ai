//! Defines the response structure for batch adding members to an organization.
//!
//! This struct provides detailed success/failure results for each email in the batch.
//! Uses 207 Multi-Status pattern to allow partial success scenarios.

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct MemberAddSuccess {
    /// Email address that was successfully added.
    #[schema(example = "user1@test.com")]
    pub email: String,

    /// User ID of the added member.
    #[schema(example = "c3d4e5f6-a7b8-9012-3456-7890abcdef12", format = "uuid", value_type = String)]
    pub user_id: uuid::Uuid,

    /// The created organization membership record.
    pub member: crate::db::organization_members::OrganizationMember,
}

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct MemberAddFailure {
    /// Email address that failed to be added.
    #[schema(example = "invalid@test.com")]
    pub email: String,

    /// Reason for the failure.
    #[schema(example = "User not found")]
    pub reason: String,
}

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AdminAddMembersResponse {
    /// List of successfully added members.
    pub success: Vec<MemberAddSuccess>,

    /// List of failed member additions with reasons.
    pub failed: Vec<MemberAddFailure>,
}
