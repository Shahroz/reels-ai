//! Represents a piece of content in the Gemini API request, containing multiple parts.
//!
//! Typically, a request contains a single `Content` item, which itself holds one or more `Part` items.
//! This structure organizes the input prompt or message components and can specify a role in conversations.
//! Uses fully qualified paths for dependencies.
//! Part of the main `Request` structure.

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct Content {
    /// Optional role for this content part, used in multi-turn conversations (e.g., "user", "model").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<crate::vendors::gemini::role::Role>,
    /// The parts that make up this content (e.g., text, inline data).
    pub parts: Vec<crate::vendors::gemini::part::Part>,
}