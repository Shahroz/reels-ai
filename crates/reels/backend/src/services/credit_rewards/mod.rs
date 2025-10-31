//! Credit rewards service module exports following one-item-per-file pattern.
//!
//! This module contains all credit rewards related functionality split
//! into individual files for better modularity and graph-based code structure.
//! Each file contains exactly one logical item (function, struct, or enum) with
//! comprehensive unit tests and proper documentation.

pub mod is_user_eligible_for_credit_rewards;

// Re-exports for backward compatibility
pub use is_user_eligible_for_credit_rewards::is_user_eligible_for_credit_rewards;
