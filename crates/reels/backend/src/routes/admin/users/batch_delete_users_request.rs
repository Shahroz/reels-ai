//! Request DTO for batch deleting users.
//!
//! This DTO defines the input shape for the admin batch user deletion endpoint.
//! Each user ID in the list will be processed with safety checks and the endpoint
//! returns detailed success/failure results for each one.

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct BatchDeleteUsersRequest {
    /// List of user IDs to delete
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440000"]))]
    pub user_ids: Vec<uuid::Uuid>,
}
