//! Defines the configuration for a research session.
//!
//! This struct holds parameters that control the behavior and limits
//! of an agent loop session, such as time limits and token thresholds.
//! It adheres to the one-item-per-file rule and uses fully qualified paths.
//! Follows formatting and documentation guidelines.

use utoipa::ToSchema; // Import ToSchema for OpenAPI documentation

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct SessionConfig {
    /// The maximum duration allowed for the session.
    pub time_limit: std::time::Duration,

    /// A potential threshold related to token usage (specific interpretation depends on implementation).
    pub token_threshold: usize,

    /// Number of recent conversation exchanges to preserve during context compaction.
    pub preserve_exchanges: usize,

    /// The initial instruction provided by the user to start the session.
    pub initial_instruction: Option<std::string::String>, // Added field

    /// Policy defining how conversation history should be compacted.
    pub compaction_policy: crate::types::compaction_policy::CompactionPolicy, // Added field

    /// Policy defining how the session's progress or final output is evaluated.
    pub evaluation_policy: crate::types::evaluation_policy::EvaluationPolicy, // Added field
}

// No tests needed for this simple data structure as per current guidelines.

impl std::default::Default for SessionConfig {
    fn default() -> Self {
        Self {
            time_limit: std::time::Duration::from_secs(3600), // Default to 1 hour
            token_threshold: 0, // Default to 0, indicating no specific threshold or to be set explicitly
            preserve_exchanges: 3, // Default to preserving 3 recent exchanges
            initial_instruction: std::option::Option::None,
            compaction_policy: crate::types::compaction_policy::CompactionPolicy::default(),
            evaluation_policy: crate::types::evaluation_policy::EvaluationPolicy::default(),
        }
    }
}