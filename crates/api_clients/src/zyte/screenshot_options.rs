//! Represents options for taking a screenshot via the Zyte API.
//!
//! Allows specifying parameters like capturing the full page.
//! Used within the main ZyteRequest structure.

//! Revision History
//! - 2025-04-15T15:06:21Z @AI: Created file based on one-item-per-file guideline.

/// Represents options for taking a screenshot.
#[derive(Debug, ::serde::Serialize)]
pub struct ScreenshotOptions {
    /// Capture the full page screenshot.
    #[serde(rename = "fullPage", skip_serializing_if = "Option::is_none")]
    pub full_page: Option<bool>,
    // TODO: Add other screenshot options if available in API docs.
}
