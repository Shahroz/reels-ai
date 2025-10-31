//! Represents a database row for a sent invitation.
//!
//! This struct models a row of data for a sent invitation, fetched from the
//! `pending_invitations` table. It's primarily used for listing invitations
//! sent by an organization.
//! Adheres to project coding standards.

// No 'use' statements as per guidelines.

/// Represents a row of data for a sent invitation.
#[derive(sqlx::FromRow, std::fmt::Debug)]
pub struct SentInvitationDbRow {
    /// The ID of the pending invitation.
    pub id: sqlx::types::Uuid,
    pub organization_id: sqlx::types::Uuid,
    pub invited_email: String,
    pub role_to_assign: String,
    pub invited_by_user_id: Option<sqlx::types::Uuid>,
    /// Timestamp of when the invitation was created/sent.
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub token_expires_at: chrono::DateTime<chrono::Utc>,
}