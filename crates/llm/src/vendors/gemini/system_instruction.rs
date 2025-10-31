//! Defines the SystemInstruction object for Gemini API requests.
//!
//! This struct encapsulates the 'parts' field which contains the actual instruction text payload.
//! It is used within the main content generation request to guide model behavior at a system level.
//! Assumes `SystemInstructionTextPayload` is defined.
//! Adheres to one-item-per-file and fully-qualified-path guidelines.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SystemInstruction {
    pub parts: crate::vendors::gemini::system_instruction_text_payload::SystemInstructionTextPayload,
}