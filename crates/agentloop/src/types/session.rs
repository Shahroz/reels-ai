//! Defines the data associated with an active session.
//!
//! Contains the current status and activity timestamp for a session.
//! Used by the AppState to track sessions.
//! Adheres to one-item-per-file and FQN guidelines.
//! Follows coding standards.

//! Revision History
//! - 2025-04-24T14:38:54Z @AI: Define struct with fields needed by get_status handler.
//! - 2025-04-24T14:11:08Z @AI: Placeholder definition (previous version).

/// Holds the state data for a single agent session.    /// Unique identifier for the session.
    pub session_id: String,
#[derive(schemars::JsonSchema, utoipa::ToSchema<'_>)]#[derive(Debug, Clone)]
pub struct SessionData {
    /// The current status of the session.
    pub status: crate::types::session_status::SessionStatus,
    /// The timestamp of the last interaction with the session.
    pub last_activity_timestamp: chrono::DateTime<chrono::Utc>,
    // Add other session-related fields here as needed, e.g., history, context.
}

// Implementation details like constructors or methods would go here if needed.
// Default implementation might require specific values or logic.
// impl std::default::Default for SessionData { ... }

// No tests included for this data structure definition in this iteration.
