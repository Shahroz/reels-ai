//! Defines the synchronous research loop orchestrator function.
//!
//! This function, `run_research_loop_sync`, is a synchronous counterpart
//! to `run_research_loop`. It executes the research process in the foreground
//! until a final answer is produced or a fatal error/timeout occurs,
//! then returns the result.
//! Adheres strictly to the project's Rust coding standards.

use crate::evaluator::research_loop::{
    check_termination_conditions::{check_termination_conditions, TerminationReason},
    handle_compaction::handle_compaction,
    initialize_loop::initialize_loop,
    process_llm_turn::process_llm_turn,
};

/// Runs the research loop synchronously for a given session until completion.
///
/// Orchestrates the research process by:
/// 1. Initializing the loop (setting status to Running).
/// 2. Repeatedly:
///    a. Fetching the latest session data.
///    b. Checking for termination conditions.
///    c. Handling conversation history compaction if needed.
///    d. Processing one turn of LLM interaction.
///    e. Handling any tool calls requested by the LLM.
/// 3. Returns the full conversation history on success, or an error on failure.
///
/// # Arguments
///
/// * `app_state` - Shared application state.
/// * `session_id` - The ID of the session to process.
///
/// # Returns
///
/// * `Ok(Vec<ConversationEntry>)` if the loop completes with a final answer.
/// * `Err(String)` if a fatal error occurs (e.g., timeout, session not found, LLM call fails).
pub async fn run_research_loop_sync(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: crate::types::session_id::SessionId,
    research_goal: std::string::String,
    mut progress_callback: Option<crate::types::progress_update::ProgressCallback>,
) -> std::result::Result<
    std::vec::Vec<crate::types::conversation_entry::ConversationEntry>,
    std::string::String,
> {
    // Initialize the loop (set status to Running)
    initialize_loop(app_state.clone(), &session_id).await?;

    loop {
        // Fetch current session data (re-fetch each iteration)
        let mut session_data =
            match crate::session::manager::get_session(app_state.clone(), &session_id).await {
                Some(data) => data,
                None => {
                    let err_msg = format!("Session {} not found.", session_id);
                    log::error!("Session {} not found during research loop.", session_id);
                    let _ = crate::session::manager::update_status(
                        app_state.clone(),
                        &session_id,
                        crate::types::session_status::SessionStatus::Error,
                    )
                    .await;
                    return std::result::Result::Err(err_msg);
                }
            };

        // If the history is empty, this is the first real turn.
        // Use the research goal as the initial user message to start the conversation.
        if session_data.history.is_empty() {
            log::info!("Session {} has an empty history. Injecting research goal as the first user message.", session_id);
            let initial_entry = crate::types::conversation_entry::ConversationEntry {
                id: uuid::Uuid::new_v4(),
                parent_id: None,
                depth: 0,
                sender: crate::types::sender::Sender::User,
                message: research_goal.clone(),
                attachments: std::vec::Vec::new(), // Infinite research starts without attachments.
                timestamp: chrono::Utc::now(),
                tools: std::vec::Vec::new(),
                tool_choice: None,
                tool_response: None,
            };

            // Persist the new entry to the session's history.
            if let Err(e) = crate::session::manager::add_conversation_entry(app_state.clone(), &session_id, initial_entry.clone()).await {
                let err_msg = format!("Failed to add initial goal to history for session {}: {}", session_id, e);
                log::error!("{}", err_msg);
                return std::result::Result::Err(err_msg);
            }

            if let Some(cb) = &mut progress_callback {
                let update = crate::types::progress_update::ProgressUpdate {
                    sender: "user".to_string(),
                    message: research_goal.clone(),
                    timestamp: chrono::Utc::now(),
                };
                cb(update).await;
            }

            // Also update the in-memory session data for the current turn.
            session_data.history.push(initial_entry);
        }

        log::debug!(
            "Session {} sync loop iteration. Status: {:?}, Goal: {:?}",
            session_id,
            session_data.status,
            session_data.research_goal
        );

        // 1. Check FATAL termination conditions
        match check_termination_conditions(&session_data, app_state.clone()).await {
            Ok(Some(reason)) => {
                let err_msg = match reason {
                    TerminationReason::Timeout => {
                        log::info!("Session {} timed out. Terminating.", session_id);
                        let _ = crate::session::manager::update_status(
                            app_state.clone(),
                            &session_id,
                            crate::types::session_status::SessionStatus::Timeout,
                        )
                        .await;
                        format!("Session {} timed out.", session_id)
                    }
                    TerminationReason::Interrupted => {
                        log::info!("Session {} interrupted. Terminating.", session_id);
                        let _ = crate::session::manager::update_status(
                            app_state.clone(),
                            &session_id,
                            crate::types::session_status::SessionStatus::Interrupted,
                        )
                        .await;
                        format!("Session {} interrupted.", session_id)
                    }
                    TerminationReason::AlreadyTerminal(status) => {
                        format!("Session {} loop entered with already terminal status: {:?}.", session_id, status)
                    }
                    TerminationReason::ConfigError(e) => {
                        log::error!("Session {} configuration error: {}. Terminating.", session_id, e);
                         let _ = crate::session::manager::update_status(
                            app_state.clone(),
                            &session_id,
                            crate::types::session_status::SessionStatus::Error,
                        ).await;
                        format!("Configuration error: {}", e)
                    }
                };
                return std::result::Result::Err(err_msg);
            }
            Ok(None) => { /* No fatal condition, continue */ }
            Err(e) => {
                log::error!("Fatal error during termination check for session {}: {}", session_id, e);
                let _ = crate::session::manager::update_status(app_state.clone(), &session_id, crate::types::session_status::SessionStatus::Error).await;
                return std::result::Result::Err(format!("Termination check failed: {}", e));
            }
        }

        // 2. Handle Compaction
        match handle_compaction(&session_data, app_state.clone(), &session_id).await {
            Ok(Some(updated_data)) => {
                session_data = updated_data;
            }
            Ok(None) => { /* No compaction or non-fatal issue */ }
            Err(e) => {
                log::error!("Fatal error during compaction for session {}: {}", session_id, e);
                let _ = crate::session::manager::update_status(app_state.clone(), &session_id, crate::types::session_status::SessionStatus::Error).await;
                return std::result::Result::Err(format!("Compaction failed: {}", e));
            }
        }

        // 3. Process LLM turn
        match process_llm_turn(&session_data, app_state.clone(), &session_id).await {
            Ok(llm_response) => {
                if let Some(cb) = &mut progress_callback {
                    let update = crate::types::progress_update::ProgressUpdate {
                        sender: "agent".to_string(),
                        message: llm_response.user_answer.clone(),
                        timestamp: chrono::Utc::now(),
                    };
                    cb(update).await;
                }
                log::debug!("run_research_loop_sync: process_llm_turn returned Ok. Response actions: {:?}", llm_response.actions);
                if !llm_response.actions.is_empty() {
                    if let Some(cb) = &mut progress_callback {
                        let tool_count = llm_response.actions.len();
                        let update = crate::types::progress_update::ProgressUpdate {
                            sender: "tool".to_string(),
                            message: format!("Using {} tool(s)", tool_count),
                            timestamp: chrono::Utc::now(),
                        };
                        cb(update).await;
                    }
                    if let Err(tool_err) = crate::evaluator::research_loop::handle_tool_calls::handle_tool_calls(
                        &llm_response,
                        app_state.clone(),
                        session_id.clone(),
                    ).await {
                        log::error!("Error handling tool calls for session {}: {}", session_id, tool_err);
                    }
                }

                if llm_response.is_final {
                    log::info!("Session {}: Final answer received in sync mode. Completing.", session_id);
                    if let Err(e) = crate::session::manager::update_status(
                        app_state.clone(),
                        &session_id,
                        crate::types::session_status::SessionStatus::Completed,
                    ).await {
                       log::error!("Failed to update session {} status to Completed: {}", session_id, e);
                    }

                    if let Some(final_session_data) = crate::session::manager::get_session(app_state.clone(), &session_id).await {
                        return std::result::Result::Ok(final_session_data.history);
                    } else {
                        let err_msg = format!("Could not retrieve session {} after completion.", session_id);
                        log::error!("{}", err_msg);
                        return std::result::Result::Err(err_msg);
                    }
                }
            }
            Err(llm_err) => {
                log::error!("LLM processing failed for session {}: {}", session_id, llm_err);
                let _ = crate::session::manager::update_status(app_state.clone(), &session_id, crate::types::session_status::SessionStatus::Error).await;
                return std::result::Result::Err(format!("LLM turn failed: {}", llm_err));
            }
        }
    }
}
