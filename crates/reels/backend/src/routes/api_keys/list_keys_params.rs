//! Defines query parameters for the list api keys endpoint.

use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

/// Query parameters for listing api keys.
#[derive(Debug, Deserialize, IntoParams, ToSchema, Clone)]
pub struct ListApiKeysParams {
    /// Search api keys by user email (case-insensitive, partial match).
    pub search: Option<String>,
}
