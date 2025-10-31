//! Organizes all database query functions related to research conversations.
//!
//! This module provides a central place for database interactions concerning
//! the `research_conversations` table, following the one-item-per-file
//! and FQN guidelines.

pub mod create_research_conversation;
pub mod delete_research_conversation;
pub mod get_research_conversation_by_id;
pub mod list_research_conversations_by_document_id;
pub mod list_research_conversations_by_user_id;
pub mod update_research_conversation;