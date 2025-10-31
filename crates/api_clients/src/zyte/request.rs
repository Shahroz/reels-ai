//! Represents the main request payload for the Zyte API extract endpoint.
//!
//! Defines the structure for specifying the URL and various extraction options
//! like requesting HTML, screenshots, or performing browser actions.
//! Includes nested structures defined in separate files.

//! Revision History
//! - 2025-04-15T15:06:21Z @AI: Created file based on one-item-per-file guideline.

/// Represents the main request payload for the Zyte API.
#[derive(Debug, serde::Serialize)]
pub struct ZyteRequest {
    /// The URL to fetch.
    pub url: String,
    /// Request the raw HTTP response body (base64 encoded).
    #[serde(rename = "httpResponseBody", skip_serializing_if = "Option::is_none")]
    pub http_response_body: Option<bool>,
    /// Request the rendered HTML content from the browser.
    #[serde(rename = "browserHtml", skip_serializing_if = "Option::is_none")]
    pub browser_html: Option<bool>,
    /// Request a screenshot (base64 encoded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<bool>,
    /// Options for the screenshot request.
    #[serde(rename = "screenshotOptions", skip_serializing_if = "Option::is_none")]
    pub screenshot_options: Option<crate::zyte::screenshot_options::ScreenshotOptions>,
    /// A list of browser actions to perform.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<crate::zyte::action::Action>>,
    // TODO: Add other Zyte API parameters as needed.
}
