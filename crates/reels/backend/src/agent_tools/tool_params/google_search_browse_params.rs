//! Defines the parameters for the 'google_search_browse' tool.
//!
//! This struct holds either a search query for Google or a URL to browse directly,
//! along with a query to process the resulting content.

/// Parameters for the 'google_search_browse' tool.
#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    std::default::Default,
    schemars::JsonSchema,
    utoipa::ToSchema
)]
pub struct GoogleSearchBrowseParams {
    /// A search query to be executed using Google Search to find relevant websites.
    /// Either `search_query` or `url` must be provided.
    #[schema(example = "What is the capital of France?")]
    pub search_query: Option<std::string::String>,
    /// The specific URL of a website to browse.
    /// Either `search_query` or `url` must be provided.
    #[schema(example = "https://en.wikipedia.org/wiki/France")]
    pub url: Option<std::string::String>,
    /// The query to process the website content with, to summarize or extract key information.
    #[schema(example = "Extract the capital city from the text.")]
    pub extraction_query: std::string::String,
}