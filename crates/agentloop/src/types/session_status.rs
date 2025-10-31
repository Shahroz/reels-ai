//! Defines the possible statuses of a session.
//!
//! Represents the lifecycle state of an agent loop session.
//! Used within SessionData and StatusResponse.
//! Adheres to one-item-per-file and FQN guidelines.
//! Follows coding standards.

//! Revision History
//! - 2025-04-24T15:58:49Z @AI: Modify Running variant to include optional progress string.
//! - 2025-04-24T14:38:54Z @AI: Initial definition based on get_status handler needs.

use schemars::JsonSchema;
use utoipa::ToSchema;

/// Represents the status of an agent session.
// Note: Using fully qualified paths for serde derive attributes for clarity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, JsonSchema, ToSchema)]
pub enum SessionStatus {
    /// The session is initializing or waiting to start processing.
    Pending,
    /// The session is actively processing the user request.
    /// Includes optional progress information.
    Running { progress: std::option::Option<std::string::String> },
    /// The session has completed successfully.
    Completed,
    /// The session encountered an error during processing.
    Error,
    /// The session has provided an answer and is awaiting further user input.
    AwaitingInput,
    /// The session was interrupted by user request.
    Interrupted,
    /// The session timed out due to inactivity.
    Timeout,
}

// No tests included for this simple enum definition. If tests were needed,
// they would be placed in a #[cfg(FALSE)] mod tests { ... } block below.

impl std::default::Default for SessionStatus {
    /// Returns the default status for a session, which is `Pending`.
    fn default() -> Self {
        SessionStatus::Pending
    }
}