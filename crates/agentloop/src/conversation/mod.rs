//! Manages conversation state, history, and interactions.
//!
//! This module defines the core structures and submodules related to
//! handling conversations, including history representation, prompt generation,
//! streaming responses, and history compaction. It re-exports key types.

pub mod compaction;
pub mod conversation_history; // Define the new module
// pub mod final_answer; // Removed
pub mod prompt; // Add the prompt module declaration
pub mod stream;

// Re-export the central type alias for convenience and accessibility.
pub use self::conversation_history::ConversationHistory;
pub use self::prompt::build_llm_prompt; // Correct the re-export path
// pub use self::final_answer::generate_final_answer; // Removed