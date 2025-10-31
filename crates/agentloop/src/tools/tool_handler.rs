//! Defines the type alias for agent tool handler functions.
//!
//! This alias represents the signature for asynchronous functions that handle
//! specific agent tools, taking strongly-typed parameters, application state, and session ID.
//! Adheres strictly to the project's Rust coding standards.
//! Follows the "one item per file" rule.

// Required for Future, Pin, Box - Used via FQN
// Required types - Used via FQN

/// Type alias for a tool handler function.
///
/// Tool handlers are asynchronous functions that take parameters (as raw JSON),
/// the shared application state, and the current session ID. They return a result
/// containing a string output or an error string.
pub type ToolHandler = fn(
    tool_choice: crate::types::tool_choice::ToolChoice,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: crate::types::session_id::SessionId,
) -> std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = Result<(crate::types::full_tool_response::FullToolResponse, crate::types::user_tool_response::UserToolResponse), String>> + Send>>; // Ensure Send for async tasks