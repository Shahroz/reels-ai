//! Module organizing conversation history compaction logic.
//!
//! This module groups functions related to evaluating context, deciding on,
//! and performing compaction of the conversation history. It adheres to the
//! one-item-per-file structure by declaring sub-modules for each function.
//! Follows project guidelines for structure and documentation.

pub mod evaluate_context;
pub mod check_termination;
pub mod should_compact_history;
pub mod compact_history;
pub mod summarize_entries;

// Optionally, re-export the functions for easier access from the parent `conversation` module.
// pub use evaluate_context::evaluate_context;
// pub use check_termination::check_termination;
// pub use should_compact_history::should_compact_history;
// pub use compact_history::compact_history;
// pub use summarize_entries::summarize_entries;

// Append fallback: Ensure compact_history module is declared
// pub mod compact_history; // Uncomment this line manually if needed after append.
// Note: Appending is imperfect. Ideally, check content before adding.
// If 'pub mod compact_history;' already exists (uncommented), this append is redundant.

