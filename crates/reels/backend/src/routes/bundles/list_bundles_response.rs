//! Defines the response structure for listing expanded bundles.
//!
//! This structure is used when API endpoints return a list of `ExpandedBundle` objects,
//! typically including pagination information like the total count of available items.
//! Adheres to Rust coding standards, including one item per file and file preamble.

/// Response payload for an API endpoint that lists multiple `ExpandedBundle` items.
#[derive(serde::Serialize, utoipa::ToSchema, std::fmt::Debug)]
pub struct ListExpandedBundlesResponse {
    /// A list of `ExpandedBundle` items.
    pub items: std::vec::Vec<crate::types::expanded_bundle::ExpandedBundle>,
    /// The total number of `ExpandedBundle` items available, matching the query criteria.
    #[schema(example = 100)]
    pub total_count: i64,
}