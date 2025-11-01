//! Defines parameter structures for Reels agent tools.
//!
//! Each file in this module defines the parameters for a specific tool,
//! adhering to the one-item-per-file guideline. These structures are used
//! for strong typing within Reels tool handlers and for generating
//! JSON schemas for AgentLoop.
//!
//! Only parameters for reel generation related tools are kept.

pub mod browse_with_query_params; // Used by generate_reel
pub mod generate_reel_params;
