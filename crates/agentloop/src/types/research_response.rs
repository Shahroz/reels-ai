//! Defines the response structure returned after creating a research session.
//!
//! This structure contains the unique identifier for the newly created session
//! and its initial status, typically 'Pending' or 'InProgress'.
//! It confirms to the client that the session creation request was accepted.
//! Follows the one-item-per-file structure.

/// Response after creating a research session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResearchResponse {
    /// The unique identifier assigned to the new session.
    pub session_id: crate::types::SessionId,
    /// The initial status of the newly created session.
    pub status: crate::types::SessionStatus,
}
