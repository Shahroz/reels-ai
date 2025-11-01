//! Application configuration structure.
//! 
//! Note: AgentLoop uses in-memory storage via AppState - no database required.
#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub server_address: String,
    pub evaluator_sleep_seconds: u64,
    pub session_timeout_seconds: u64,
    pub llm_config: crate::config::llm_config::LlmConfig,
    pub compaction_policy: crate::types::compaction_policy::CompactionPolicy,
    pub max_conversation_length: usize,
}
