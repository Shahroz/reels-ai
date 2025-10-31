//! Defines the request structure for terminating a research session.
//!
//! This struct contains the necessary information to identify the session
//! that needs to be terminated. It is used in API calls related
//! to stopping ongoing sessions forcefully.
//! Ensures the session ID is provided for termination requests.

use serde::{Deserialize, Serialize};

/// Request to force-terminate a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminationRequest {
    /// The unique identifier of the session to be terminated.
    pub session_id: crate::types::SessionId,
}
