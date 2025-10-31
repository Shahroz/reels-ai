//! Placeholder for CompactionPolicy struct.

use utoipa::ToSchema; // Import ToSchema for OpenAPI documentation

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[derive(ToSchema)] // Add ToSchema derive
pub struct CompactionPolicy {
    pub keep_last: usize,
    pub summary_length: usize,
}
