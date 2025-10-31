//! Request structure for updating a user's status via admin endpoint.
//!
//! This struct defines the payload for status updates. The status field must be
//! one of the valid user status values: active, trial, expired, or cancelled.
//! Used by admin users to change user account status without modifying other fields.
//! All status changes are logged to the audit trail.

/// Request payload for updating user status
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateUserStatusRequest {
    /// The new status for the user account (active, trial, expired, or cancelled)
    #[schema(example = "active")]
    pub status: crate::db::user_status::UserStatus,
}

