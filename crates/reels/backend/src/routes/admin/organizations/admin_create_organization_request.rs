//! Defines the request body for creating an organization via admin endpoint.
//!
//! This struct specifies the required fields for an admin to create a new organization
//! with a specified owner. The owner will automatically be added as a member with owner role.

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AdminCreateOrganizationRequest {
    /// Name of the organization to create.
    #[schema(example = "Acme Corporation")]
    pub name: String,

    /// UUID of the user who will own this organization.
    #[schema(example = "b2c3d4e5-f6a7-8901-2345-67890abcdef1", format = "uuid", value_type = String)]
    pub owner_user_id: uuid::Uuid,
}
