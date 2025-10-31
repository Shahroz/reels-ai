//! Represents a piece of persisted context within a session.
//!
//! Stores the actual content, an optional source identifier,
//! and the timestamp when it was recorded.
//! Used to maintain relevant information gathered during agent operation.
//! Adheres to the one-item-per-file guideline.

// Note: `use serde::{...}` is included based on guideline examples for derive macros.
// Import ToSchema for OpenAPI documentation generation
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};

/// A piece of persisted context.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContextEntry {
    /// The textual content of the context entry.
    pub content: std::string::String,
    /// Optional identifier for where this context originated (e.g., URL, file).
    pub source: std::option::Option<std::string::String>,
    /// Timestamp indicating when this context entry was created or recorded.
    // Fully qualified path used as per guidelines. Timestamp is defined in crate::types::mod.rs
    pub timestamp: chrono::DateTime<chrono::Utc>
}
