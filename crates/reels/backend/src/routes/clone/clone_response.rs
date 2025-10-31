//! Defines the response body for the clone endpoint.
//!
//! This struct represents the JSON payload returned by the clone API, including styled content and the request ID.
//!
//! Revision History
//! - 2025-04-21T15:01:17Z @AI: Refactored CloneResponse into its own file.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct CloneResponse {
    #[schema(example = "<html> ... styled content ...</html>")]
    pub styled_content: String,
    #[schema(example = 123)]
    pub request_id: i32,
}
