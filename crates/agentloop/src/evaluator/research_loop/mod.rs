//! Organizes the components of the main research loop logic.
//!
//! This module breaks down the large `run_research_loop` function into
//! smaller, more manageable units, each handling a specific part of the loop's
//! functionality (e.g., initialization, termination checks, compaction, LLM calls, tool handling).
//! Adheres strictly to the project's Rust coding standards.

pub mod check_sufficiency_for_answer;
pub mod initialize_loop;
pub mod check_termination_conditions;
pub mod handle_compaction;
pub mod process_llm_turn;
pub mod handle_tool_calls;
// Add fetch_session_data, handle_llm_error if created as separate files.