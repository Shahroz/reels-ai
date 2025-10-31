//! Defines the main research loop orchestrator function.
//!
//! This refactored version of `run_research_loop` coordinates calls to
//! specialized functions within the `research_loop` submodule to handle
//! different phases of the loop (initialization, termination checks, compaction,
//! LLM calls, tool handling).
//! Adheres strictly to the project's Rust coding standards (FQN, one item per file principle applied via submodules).

//! Revision History
//! - 2025-04-25T09:26:14Z @AI: Fix session_id type mismatch and FinalAnswerOutcome handling.
//! - 2025-04-25T08:42:52Z @AI: Modify logic to update status based on generate_and_send_final_answer result.
//! - 2025-04-24T20:57:27Z @AI: Refactor into smaller functions within research_loop submodule.
//! - 2025-04-24T19:48:16Z @AI: Modify WebSocket sending in Ok arm per instruction (log, extract, use do_send, drop lock early).
//! - 2025-04-24T19:12:42Z @AI: Refactor final answer generation to occur *before* breaking loop for Timeout/Completed.
//! - 2025-04-24T17:18:54Z @AI: Fix syntax errors related to misplaced update_status calls.
//! - 2025-04-24T16:14:28Z @AI: Integrate tool dispatch and conversation compaction logic. Address mutable history assumption for compaction.
//! - 2025-04-24T16:12:17Z @AI: Add termination checks (LLM, timeout, interrupt) and agent history logging. Remove dev break.
//! - 2025-04-24T16:10:40Z @AI: Initial implementation with basic loop structure.

use crate::evaluator::research_loop::{
    check_termination_conditions::{check_termination_conditions, TerminationReason},
    handle_compaction::handle_compaction,
    initialize_loop::initialize_loop,
    process_llm_turn::process_llm_turn,
};

/// Runs the main research loop for a given session.
///
/// Orchestrates the research process by:
/// 1. Initializing the loop (setting status to Running).
/// 2. Repeatedly:
///    a. Fetching the latest session data.
///    b. Checking for termination conditions.
///    c. Handling conversation history compaction if needed.
///    d. Processing one turn of LLM interaction.
///    e. Handling any tool calls requested by the LLM.
/// 3. Handling loop termination and potential errors.
///
/// # Arguments
///
/// * `app_state` - Shared application state.
/// * `session_id` - The ID of the session to process.
///
/// # Returns
///
/// * `Ok(())` if the loop completes normally (due to termination conditions).
/// * `Err(String)` if a fatal error occurs (e.g., session not found, initial status update fails, LLM call fails critically).

/// Runs the main research loop for a given session.
pub async fn run_research_loop(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: crate::types::session_id::SessionId,
    mut progress_callback: Option<crate::types::progress_update::ProgressCallback>,
) -> std::result::Result<(), std::string::String> {
    // Initialize the loop (set status to Running)
    initialize_loop(app_state.clone(), &session_id).await?;

    loop {
        // Fetch current session data (re-fetch each iteration)
        let mut session_data =
            match crate::session::manager::get_session(app_state.clone(), &session_id).await
            {
                Some(data) => data,
                None => {
                    log::error!("Session {} not found during research loop.", session_id);
                    // Attempt to update status before returning error - Adding missing app_state argument
                    let _ = crate::session::manager::update_status(
                        app_state.clone(),
                        &session_id,
                        crate::types::session_status::SessionStatus::Error,
                    )
                    .await;
                    return std::result::Result::Err(format!("Session {} not found.", session_id));
                }
            };

        // Log the current goal at the start of each iteration, especially useful after restart
        log::debug!(
            "Session {} loop iteration. Status: {:?}, Goal: {:?}",
            session_id, session_data.status, session_data.research_goal
        );

        // 1. Check FATAL termination conditions (Timeout, Interrupted, Error, ConfigError)
        match check_termination_conditions(&session_data, app_state.clone()).await {
            Ok(Some(reason)) => {
                match reason {
                    // Handle only fatal conditions here
                    TerminationReason::Timeout => {
                        log::info!("Session {} timed out. Terminating.", session_id);
                        if let Err(e) = crate::session::manager::update_status(
                            app_state.clone(),
                            &session_id,
                            crate::types::session_status::SessionStatus::Timeout,
                        )
                        .await
                        {
                           log::error!("Failed to update session {} status to Timeout: {}", session_id, e);
                        }
                    }
                    TerminationReason::Interrupted => {
                        log::info!("Session {} interrupted. Terminating.", session_id);
                        // Status should already be Interrupted, no update needed generally,
                        // but confirming doesn't hurt.
                        if let Err(e) = crate::session::manager::update_status(
                            app_state.clone(),
                            &session_id,
                            crate::types::session_status::SessionStatus::Interrupted,
                        )
                        .await
                        {
                            log::error!("Failed to update session {} status to Interrupted: {}", session_id, e);
                        }
                    }
                    TerminationReason::AlreadyTerminal(status) => {
                        log::warn!("Session {} loop entered with already terminal status: {:?}. Terminating.", session_id, status);
                        // No status update needed
                    }
                    TerminationReason::ConfigError(e) => {
                        log::error!("Session {} configuration error: {}. Terminating.", session_id, e);
                        if let Err(update_err) = crate::session::manager::update_status(
                            app_state.clone(),
                            &session_id,
                            crate::types::session_status::SessionStatus::Error,
                        )
                        .await
                        {
                           log::error!("Failed to update session {} status to Error after config error: {}", session_id, update_err);
                        }
                    }
                };
                // Break after handling fatal condition
                return std::result::Result::Ok(());
            }
            Ok(None) => { /* No fatal condition, continue */ }
            Err(e) => {
                // Fatal error during checks (e.g., invalid session ID format)
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
                let _ = crate::session::manager::update_status(app_state.clone(), &session_id,
                    crate::types::session_status::SessionStatus::Error).await;
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
                log::debug!("run_research_loop: process_llm_turn returned Ok. Response actions: {:?}", llm_response.actions);
                // Handle tool calls
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
                // Check final flag
                if llm_response.is_final {
                    log::info!("Session {}: Final answer received. Transitioning to AwaitingInput.", session_id);
                    let _ = crate::session::manager::update_status(app_state.clone(), &session_id,
                        crate::types::session_status::SessionStatus::AwaitingInput).await;
                    break Ok(());
                }
                // Else continue loop
            }
            Err(llm_err) => {
                log::error!("LLM processing failed for session {}: {}", session_id, llm_err);
                let _ = crate::session::manager::update_status(app_state.clone(), &session_id,
                    crate::types::session_status::SessionStatus::Error).await;
                return std::result::Result::Err(format!("LLM turn failed: {}", llm_err));
            }
        }
    }
}
