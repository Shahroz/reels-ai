//! Represents the text payload within the 'parts' field of a Gemini SystemInstruction.
//!
//! This structure directly maps to the `{\"text\": \"...\"}` object found in the
//! `parts` field of a system instruction. It is used by the `SystemInstruction`
//! struct to encapsulate the instructional text.
//! Adheres to one-item-per-file and fully-qualified-path guidelines.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SystemInstructionTextPayload {
    pub text: std::string::String,
}