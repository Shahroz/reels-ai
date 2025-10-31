//! Typed errors for admin service layer operations.
//!
//! This enum provides structured error types for admin operations, allowing
//! handlers to make decisions based on error variants rather than string matching.
//! Each variant represents a specific error condition with appropriate HTTP
//! status code mapping (400 for client errors, 500 for server errors).
//!
//! Revision History:
//! - 2025-10-10: Initial creation for typed error handling in admin services.

#[derive(Debug, thiserror::Error)]
pub enum AdminServiceError {
    #[error("Organization name cannot be empty")]
    EmptyOrganizationName,

    #[error("Specified owner user does not exist")]
    OwnerUserNotFound,

    #[error("At least one field must be provided for update")]
    NoFieldsToUpdate,

    #[error("Organization not found")]
    OrganizationNotFound,

    #[error("At least one email must be provided")]
    EmptyEmailList,

    #[error("Maximum {max} emails allowed per batch, received {actual}")]
    TooManyEmails { max: usize, actual: usize },

    #[error("Cannot delete admin users")]
    CannotDeleteAdmin,

    #[error("Cannot delete self")]
    CannotDeleteSelf,

    #[error("Cannot add members to a personal organization. Personal organizations are private to their owner.")]
    CannotAddMembersToPersonalOrg,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    InternalError(#[from] anyhow::Error),
}

impl AdminServiceError {
    /// Returns true if this error should result in a 400 Bad Request response.
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::EmptyOrganizationName
                | Self::OwnerUserNotFound
                | Self::NoFieldsToUpdate
                | Self::EmptyEmailList
                | Self::TooManyEmails { .. }
                | Self::CannotDeleteAdmin
                | Self::CannotDeleteSelf
                | Self::CannotAddMembersToPersonalOrg
        )
    }

    /// Returns true if this error indicates a resource was not found (404).
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::OrganizationNotFound)
    }
}

