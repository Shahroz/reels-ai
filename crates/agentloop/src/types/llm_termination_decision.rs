//! Defines the structured response from the LLM for conversation termination decisions.
//!
//! This struct holds the LLM's judgment on whether a conversation should end,
//! based on the provided context (history, goal, etc.). It implements the necessary
//! traits for LLM interaction and serialization. Adheres to project guidelines.

// Note: Using external crate traits requires `use` or fully qualified calls.
// `JsonSchema` derive requires `schemars`. `FewShotsOutput` requires `llm`.
use llm::few_shots_traits::FewShotsOutput;
use schemars::JsonSchema; // Required for the derive macro

/// Represents the LLM's decision on whether to terminate the conversation.
#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct LlmTerminationDecision {
    /// The reasoning provided by the LLM for its decision.
    pub reasoning: std::string::String,
    /// True if the LLM determined the conversation should terminate, false otherwise.
    pub should_terminate: bool,
}

// Implement the FewShotsOutput trait for the LLM interaction.
impl FewShotsOutput<LlmTerminationDecision> for LlmTerminationDecision {
    fn few_shots() -> std::vec::Vec<LlmTerminationDecision> {
        std::vec![
            // Example 1: Termination condition met (goal achieved)
            LlmTerminationDecision {
                should_terminate: true,
                reasoning: std::string::String::from(
                    "The agent provided a summary that directly addresses the user's goal, and the user acknowledged it.",
                ),
            },
            // Example 2: Termination condition met (user request)
            LlmTerminationDecision {
                should_terminate: true,
                reasoning: std::string::String::from(
                    "The user explicitly asked to end the conversation using the keyword 'stop'.",
                ),
            },
            // Example 3: No termination condition met (ongoing discussion)
            LlmTerminationDecision {
                should_terminate: false,
                reasoning: std::string::String::from(
                    "The conversation is ongoing. The user asked a follow-up question, indicating the goal is not yet met.",
                ),
            },
            // Example 4: No termination condition met (agent needs more info)
            LlmTerminationDecision {
                should_terminate: false,
                reasoning: std::string::String::from(
                    "The agent requires clarification or further input from the user to proceed.",
                ),
            },
        ]
    }
}

// No tests needed for this type definition with FewShots.
