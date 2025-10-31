//! Defines a simplified article structure containing key extracted text content and metadata.
//!
//! This is used as the return type for functions that fetch and process web content,
//! focusing on the readable text and essential metadata, excluding the raw HTML.

//! Revision History
//! - 2025-05-02T04:59:40Z @AI: Created file for SimplifiedArticle struct.

/// Simplified representation of extracted article content and metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct SimplifiedArticle {
    /// The main title of the article.
    pub title: String,
    /// The extracted text content of the article.
    pub text_content: String,
    /// The author or byline, if available.
    pub byline: Option<String>,
    /// A short summary or excerpt, if available.
    pub excerpt: Option<String>,
    /// The name of the website or publication, if available.
    pub site_name: Option<String>,
    /// The language of the article, if available (e.g., "en").
    pub lang: Option<String>,
    /// The canonical URL of the article, if available.
    pub url: Option<String>,
    // Add other relevant fields from dom_smoothie::Article if needed
}
