//! Placeholder for EvaluationPolicy struct.
//!
//! Defines the policy for evaluating the state or output of a research session.
//! Adheres to one-item-per-file and FQN guidelines.
//! Requires further definition based on evaluation logic.

use utoipa::ToSchema; // Import ToSchema for OpenAPI documentation

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[derive(ToSchema)] // Add ToSchema derive
pub struct EvaluationPolicy {
    // Placeholder fields - adjust based on actual evaluation criteria
    pub success_threshold: f64,
    pub require_final_answer: bool,
}
