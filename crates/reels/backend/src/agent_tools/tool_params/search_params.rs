//! Defines the parameters for the Narrativ 'search' tool.
//!
//! This struct holds the search query. It's used for strong typing
//! in Narrativ's search tool handler and for schema generation.

/// Parameters for the 'search' tool.
#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    utoipa::ToSchema,
    Default
)]
pub struct SearchParams {
    /// The search query string.
    #[schema(example = "latest advancements in AI")]
    pub query: std::string::String,
    // Potentially add other search options like 'num_results', 'time_period', etc.
}