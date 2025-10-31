//! Defines the reasoning structure for OpenAI API requests.
//!
//! Contains the 'effort' field to specify the desired reasoning level.
//! Used within the main chat completion request structure.
//! Encapsulates the reasoning parameters.
//! Follows the structure observed in API examples.

// Fully qualified path for ReasoningEffort:
// ReasoningEffort -> crate::vendors::openai::reasoning_effort::ReasoningEffort

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Reasoning {
    pub effort: crate::vendors::openai::reasoning_effort::ReasoningEffort,
}
