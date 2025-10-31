use schemars::JsonSchema;
use utoipa::ToSchema;
// Assuming SessionStatus needs importing if not in prelude/core types
// use crate::types::session_status::SessionStatus; // Example if needed, but FQN used inline
// Assuming SessionId needs importing if not in prelude/core types
// use crate::types::session_id::SessionId; // Example if needed, but FQN used inline

// Ensure the utility module exists if used:
// #[path = "../../utils/serde_option_duration_as_secs.rs"] // Example if path needed
// mod serde_option_duration_as_secs; // Example if needed

// Defines the response structure for the session status endpoint.
//
// Contains the session ID, current status, and optional remaining time until timeout.
// Adheres to one-item-per-file and FQN guidelines.
// Serialization uses a custom helper for Option<Duration>.

// Revision History
// - 2025-04-24T14:44:32Z @AI: Initial definition based on get_status handler usage.

/// Response payload for the session status endpoint.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, JsonSchema, ToSchema)] // Added Debug and Clone for potential use
pub struct StatusResponse {
    /// The unique identifier of the session.
    pub session_id: String,
    /// The current status of the session.
    pub status: crate::types::session_status::SessionStatus, // Assuming SessionStatus is clonable
    /// Optional remaining time until the session expires, in seconds.
    /// Uses a custom serializer/deserializer if needed (assumed crate::utils::serde_option_duration_as_secs exists).
    pub time_remaining: std::option::Option<std::time::Duration>,
}

// Assuming crate::utils::serde_option_duration_as_secs exists and handles Option<std::time::Duration> <-> Option<u64>
// If it doesn't exist, it would need to be created or this attribute removed/adapted.

// No tests included for this data structure definition in this iteration.
// Tests would typically involve serialization/deserialization checks.

