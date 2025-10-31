//! Represents a single message in an OpenAI chat completion request.
//!
//! Each message has content (optional), a role indicating the sender,
//! and an optional name field for associating messages with specific entities
//! (like function calls or specific user identifiers).
//! Uses skip_serializing_none to omit fields that are None during serialization.

#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub content: Option<String>,
    pub role: crate::vendors::openai::role::Role, // FQN for Role
    pub name: Option<String>,
}
