//! Defines the request body for updating an organization via admin endpoint.
//!
//! This struct specifies the fields that can be updated by an admin. All fields are optional
//! to support partial updates. If owner_user_id is changed, the old owner will be demoted
//! to member and the new owner will be promoted to owner role.

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AdminUpdateOrganizationRequest {
    /// New name for the organization (optional).
    #[schema(example = "Acme Corp")]
    pub name: Option<String>,

    /// New owner user ID (optional). If provided, ownership will be transferred.
    #[schema(example = "c3d4e5f6-a7b8-9012-3456-7890abcdef12", format = "uuid", value_type = Option<String>)]
    pub owner_user_id: Option<uuid::Uuid>,
}
