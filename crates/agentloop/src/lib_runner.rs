//! Defines a library-callable entry point for running a synchronous research task.
//!
//! This function encapsulates session creation and the execution of the research loop,
//! providing a simplified interface for external callers who want to run a one-off task.
//! It adheres strictly to the project's Rust coding standards, including the use of
//! fully qualified paths and file-level documentation.

/// Runs a synchronous research task from start to finish.
///
/// This function performs the following steps:
/// 1. Creates a default session configuration.
/// 2. Sets the initial instruction from the incoming request.
/// 3. Creates a new session using the session manager.
/// 4. Executes the synchronous research loop (`run_research_loop_sync`).
/// 5. Returns the final conversation history or an error.
///
/// # Arguments
///
/// * `app_state` - The shared application state.
/// * `request` - The research request containing the user ID and instruction.
///
/// # Returns
///
/// * `Ok(Vec<ConversationEntry>)` on successful completion of the research task.
/// * `Err(String)` if any step in the process fails.
pub async fn run_research_task(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    request: crate::types::research_request::ResearchRequest,
) -> std::result::Result<
    std::vec::Vec<crate::types::conversation_entry::ConversationEntry>,
    std::string::String,
> {
    // a. Create a default session config and set the initial instruction.
    let mut config = crate::types::session_config::SessionConfig::default();
    config.initial_instruction = std::option::Option::Some(request.instruction.clone());

    // b. Call `create_session` to get a new `session_id`.
    // The user_id is passed from the request, as required by create_session.
    let session_id = crate::session::manager::create_session(
        request.user_id,
        app_state.clone(),
        config,
        request.organization_id,
    )
    .await;

    // c. Call `run_research_loop_sync` with the `app_state` and the new `session_id`.
    let result = crate::evaluator::run_research_loop_sync::run_research_loop_sync(
        app_state,
        session_id,
        request.instruction,
        None, // No progress callback for lib_runner
    )
    .await;

    // d. Return the result from `run_research_loop_sync`.
    result
}

// Per guidelines, tests would be here if specified.
// #[cfg(test)]
// mod tests {
//     // Tests would go here.
// }