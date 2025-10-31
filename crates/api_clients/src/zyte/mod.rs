//! Module containing all components for interacting with the Zyte API.
//!
//! Includes the client, request/response structures, error types,
//! and specific functionalities like sitemap processing and page fetching.

//! Revision History
//! - 2025-04-24T08:26:22Z @AI: Created module file during reorganization.

pub mod article;
pub mod action;
pub mod client;
pub mod simplified_article; // Added
pub mod error;
pub mod fetch_and_parse_sitemap;
pub mod fetch_and_save_text;
pub mod fetch_browser_html; // Added
pub mod fetch_clean_and_save_page; // Kept existing
pub mod filter_sitemap_urls; // Kept existing
pub mod request;
pub mod response;
pub mod screenshot_options;
pub mod sitemap_processor;