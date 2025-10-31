//! Defines the type alias for representing conversation history.
//!
//! This centralizes the definition of `ConversationHistory` as a vector
//! of `crate::types::conversation_entry::ConversationEntry` items, ensuring consistency.
//! Uses fully qualified paths as per project guidelines.
//! Adheres to the one-item-per-file rule.

pub type ConversationHistory = Vec<crate::types::conversation_entry::ConversationEntry>;

// Note: No tests typically needed for a simple type alias definition.
