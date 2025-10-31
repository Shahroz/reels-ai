//! Defines the `OutputFormat` enum for specifying LLM response serialization formats.
//!
//! This enum is used by the `llm_typed` function to determine how to format
//! prompts regarding expected output structure and how to parse the LLM's response.
//! It supports common serialization formats like JSON, YAML, and TOML, as well as
//! other structured text formats.

use serde::{Serialize, Deserialize}; // Allowed for derive macros as per instruction

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Yaml,
    TOML,
    XML,
    Tags,
}