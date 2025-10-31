//! Defines HTTP request handlers for the AgentLoop service.
//!
//! Organizes the different API endpoints like starting research,
//! getting status, posting messages, terminating sessions, and
//! streaming conversation events. Each handler resides in its own file.

//! Revision History
//! - 2025-05-12T12:49:39Z @AI: Add get_session_state and load_session_state handlers.
//! - 2025-04-24T12:45:12Z @AI: Initial module setup based on required handlers.

pub mod conversation_stream;
pub mod get_status;
pub mod post_message;
pub mod start_research;
pub mod terminate_session;
pub mod get_session_state; // Added handler for getting session state
pub mod load_session_state; // Added handler for loading session state

pub mod run_research_sync;
