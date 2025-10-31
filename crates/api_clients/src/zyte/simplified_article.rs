//! Represents a simplified version of an article extracted from HTML.
//!
//! This struct holds key information like title, content (HTML), author, etc.,
//! extracted using readability algorithms like dom_smoothie.

//! Revision History
//! - 2025-05-02T12:11:38Z @AI: Created file, moved struct from fetch_browser_html.rs, changed text_content to content.

/// Simplified article data extracted from HTML.
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)] // Added serde derive
pub struct SimplifiedArticle {
    pub title: String,
    /// The main content of the article, possibly containing HTML tags including links.
    pub content: String, // Changed from text_content
    pub byline: Option<String>,
    pub excerpt: Option<String>,
    pub site_name: Option<String>,
    pub lang: Option<String>,
    pub url: Option<String>, // URL from dom_smoothie extraction
}
