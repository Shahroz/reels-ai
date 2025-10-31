//! Defines the structure for feedback from the context evaluator.
//!
//! This struct encapsulates the analysis performed on the conversation context,
//! providing insights into its relevance, suggestions for next steps, and whether
//! the context requires updating or augmentation.
//! Adheres to one-item-per-file, FQN, and documentation guidelines.

//! Revision History
//! - 2025-04-24T14:48:38Z @AI: Updated struct definition based on usage in evaluate_context.rs.
//! - 2025-04-24T14:44:32Z @AI: Created placeholder file.

use schemars::JsonSchema;
use utoipa::ToSchema;

// Note: serde derives are handled by the derive macro itself.
// No need for FQN like `serde::ser::Serialize` here.
#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, ToSchema, JsonSchema)]
pub struct ContextEvaluatorFeedback {
    /// A score indicating the relevance or sufficiency of the current context.
    /// Ranges from 0.0 (insufficient) to 1.0 (sufficient).
    pub relevance_score: f64,

    /// A list of suggested next steps for the agent based on the context analysis.
    pub suggestions: std::vec::Vec<std::string::String>,

    /// Flag indicating whether the context is deemed insufficient or needs updating.
    /// True if updates/clarifications are needed, false otherwise.
    pub needs_update: bool,
}

// No tests needed for a simple data structure definition.
// Tests would typically reside with the code *using* this struct.
