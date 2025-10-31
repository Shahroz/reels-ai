//! Provides shared utility functions for the agentloop crate.
//!
//! This module aggregates various helper functions and sub-modules
//! that offer common, reusable logic across different parts of the
//! agentloop application. It helps in maintaining a clean and organized
//! codebase by centralizing utility code.
//! Organizes utilities such as message formatting and custom serialization helpers.

pub mod message_formatter;
pub mod serde_option_duration_as_secs;