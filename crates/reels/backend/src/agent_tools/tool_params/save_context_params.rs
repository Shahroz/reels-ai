//! Defines the parameters for the Narrativ 'save_context' tool.
//!
//! This struct holds the content to be saved and an optional source identifier.
//! It's used for strong typing in Narrativ's save_context tool handler and for schema generation.

/// Parameters for the 'save_context' tool.
#[derive(
    std::fmt::Debug,
    std::clone::Clone,
   std::default::Default,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    utoipa::ToSchema
)]
pub struct SaveContextParams {
    /// The textual content to be saved to the agent's context.
    #[schema(example = "This is an important piece of information.")]
    pub content: std::string::String,

    /// An optional source identifier for the content (e.g., URL, document name).
    #[schema(example = "https://example.com/article", value_type = Option<String>)]
    pub source: std::option::Option<std::string::String>,
}