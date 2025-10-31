//! Defines the request body for batch adding members to an organization.
//!
//! This struct specifies the list of email addresses to add as members and their role.
//! The endpoint will attempt to add all users and return detailed success/failure results.

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AdminAddMembersRequest {
    /// List of email addresses to add as members.
    #[schema(example = json!(["user1@test.com", "user2@test.com"]))]
    pub emails: Vec<String>,

    /// Role to assign to the new members (defaults to "member").
    #[schema(default = "member", example = "member")]
    pub role: Option<String>,
}
