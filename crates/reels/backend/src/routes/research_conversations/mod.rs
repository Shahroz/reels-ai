//! Module organizing request/response models and handlers for research conversations.
//!
//! This module defines the data structures used in API requests/responses
//! related to research conversations and includes the handlers that interact
//! with the database for these operations.

pub mod create_research_conversation_request;
pub mod update_research_conversation_state_request;
pub mod create_conversation;
pub mod get_conversation;
pub mod update_conversation_state;
pub mod configure_research_conversation_routes;
pub mod list_conversations_by_document;
pub mod delete_conversation;
