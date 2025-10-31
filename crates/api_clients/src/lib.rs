//! API client library for Zyte and Serper services.
//!
//! This library provides modules and functions to interact with:
//! - Zyte API: For web scraping, data extraction, sitemap processing, single page text extraction.
//! - Serper API: For performing Google searches.
//!
//! It re-exports key functions mimicking the CLI tool's capabilities.

//! Revision History
//! - 2025-04-24T08:35:52Z @AI: Re-exported functions aligned with CLI commands.
//! - 2025-04-24T08:31:37Z @AI: Re-exported key components based on instruction.
//! - 2025-04-24T08:26:22Z @AI: Reorganized library into zyte and serper modules.
//! - 2025-04-15T15:06:21Z @AI: Updated module declarations for one-item-per-file structure.

pub mod serper;
pub mod zyte;
pub mod webflow;

// Re-export key clients and errors (optional but potentially useful)
pub use crate::zyte::client::ZyteClient;
pub use crate::zyte::error::ZyteError;
pub use crate::webflow::WebflowClient;
pub use crate::webflow::WebflowError;
// pub use crate::serper::error::SerperError; // Assuming serper::error defines SerperError - uncomment if needed

// Re-export functions aligned with CLI commands
pub use crate::zyte::sitemap_processor::process_sitemap; // For Sitemap command
pub use crate::zyte::fetch_and_save_text::fetch_and_save_text; // For SinglePage command
pub use crate::serper::client::search as serper_search; // For Serper command
