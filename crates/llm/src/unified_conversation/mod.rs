//! Defines and re-exports unified data structures for representing conversations.
//!
//! This module provides a common way to structure conversational data (roles, messages, content)
//! that can be used across different LLM vendor integrations (Claude, Gemini, OpenAI, Replicate).
//! It adheres to the one-item-per-file rule and uses fully qualified paths.
//! Public items are re-exported for convenience.

// Declare modules for each item/file
pub mod unified_conversation;
pub mod unified_message;
pub mod unified_message_content;
pub mod unified_role;

// Re-export public items for external use
pub use self::unified_conversation::UnifiedConversation;
pub use self::unified_message::UnifiedMessage;
pub use self::unified_message_content::UnifiedMessageContent;
pub use self::unified_role::UnifiedRole;