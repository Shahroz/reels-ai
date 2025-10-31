//! Request DTO for batch creating users.
//!
//! This DTO defines the input shape for the admin batch user creation endpoint.
//! Each email in the list will be processed and the endpoint returns detailed
//! success/failure results for each one.

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct BatchCreateUsersRequest {
    /// List of email addresses for users to create
    #[schema(example = json!(["user1@example.com", "user2@example.com"]))]
    pub emails: Vec<String>,
}
