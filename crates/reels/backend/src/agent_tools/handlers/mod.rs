//! Defines handlers for Reels agent tools.
//!
//! Each file in this module implements the logic for a specific tool,
//! taking strongly-typed parameters and returning results compatible with
//! AgentLoop's `FullToolResponse` and `UserToolResponse`.
//!
//! Only tools related to reel generation are kept:
//! - generate_reel: Core tool for generating reels
//! - browse_with_query: Used internally by generate_reel to fetch product information

pub mod handle_reels_browse_with_query; // Used by generate_reel
pub mod handle_generate_reel;
