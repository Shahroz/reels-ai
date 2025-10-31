//! Represents the data structure for a pending invitation response.
//!
//! This struct is typically used in API responses to provide detailed information
//! about a pending invitation, including organization details and inviter's email.
//! It combines data from the `pending_invitations` table with related tables.
//! Adheres to project coding standards.

// No 'use' statements as per guidelines.

/// Data structure for a pending invitation response.
#[derive(
    std::fmt::Debug,
    serde::Serialize,
    serde::Deserialize,
    sqlx::FromRow,
    utoipa::ToSchema,
)]
pub struct PendingInvitationResponse {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type = String)]
    pub pending_invitation_id: sqlx::types::Uuid,
    #[schema(example = "b2c3d4e5-f6a7-8901-2345-67890abcdef1", format = "uuid", value_type = String)]
    pub organization_id: sqlx::types::Uuid,
    #[schema(example = "Awesome Inc.")]
    pub organization_name: String,
    #[schema(example = "invitee@example.com")]
    pub invited_email: String,
    #[schema(example = "member")]
    pub role_to_assign: String,
    #[schema(example = "jwt_token_string")]
    pub invitation_token: String,
    #[schema(value_type = String, format = "date-time", example = "2024-05-23T10:00:00Z")]
    pub token_expires_at: chrono::DateTime<chrono::Utc>,
    #[schema(example = "c3d4e5f6-a7b8-9012-3456-7890abcdef12", format = "uuid", value_type = Option<String>)]
    pub invited_by_user_id: Option<sqlx::types::Uuid>,
    #[schema(example = "inviter@example.com", value_type = Option<String>)]
    pub inviter_email: Option<String>,
    #[schema(value_type = String, format = "date-time", example = "2024-05-22T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}