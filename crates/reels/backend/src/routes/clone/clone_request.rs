//! Defines the request body for the clone endpoint.
//!
//! This struct represents the JSON payload for the clone API, including URL, optional content, and operation mode.
//!
//! Revision History
//! - 2025-04-21T15:01:17Z @AI: Refactored CloneRequest into its own file.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct CloneRequest {
    #[schema(example = "https://example.com")]
    pub url: String,
    #[schema(example = "<h1>Hello</h1><p>Style me</p>")]
    pub content_to_style: Option<String>,
    #[schema(example = "fast")]
    pub mode: String,
}
