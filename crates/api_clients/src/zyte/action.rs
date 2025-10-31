//! Represents an action to be performed by the browser in a Zyte API request.
//!
//! Defines actions like scrolling the page.
//! Used as part of a list within the main ZyteRequest structure.

//! Revision History
//! - 2025-04-15T15:06:21Z @AI: Created file based on one-item-per-file guideline.

/// Represents an action to be performed by the browser.
#[derive(Debug, ::serde::Serialize)]
pub struct Action {
    /// The name of the action (e.g., "scrollBottom").
    pub action: String,
    // TODO: Add other action parameters if needed.
}
