//! Defines the possible roles in an OpenAI chat completion message.
//!
//! Roles specify the sender of a message, typically System, User, or Assistant.
//! This enum is used within the Message struct to structure conversations.
//! It directly maps to the roles accepted by the OpenAI API.
//! Serialization uses lowercase names.

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}
