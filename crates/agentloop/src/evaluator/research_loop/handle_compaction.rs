//! Manages the conversation history compaction process within the research loop.
//!
//! Checks if compaction is necessary based on the history and policy,
//! performs compaction if needed, and handles potential errors or the case
//! where the session disappears during compaction.
//! Adheres to the one-item-per-file guideline and uses FQNs.

/// Checks if compaction is needed and performs it, returning updated session data if successful.
///
/// # Arguments
/// * `session_data` - The current session data (used for checking).
/// * `app_state` - Shared application state.
/// * `session_id` - The ID of the session.
///
/// # Returns
/// * `Ok(Some(SessionData))` if compaction occurred and session data was re-fetched successfully.
/// * `Ok(None)` if no compaction was needed or if compaction failed non-fatally.
/// * `Err(String)` if the session disappears during or after compaction.
pub async fn handle_compaction(
    session_data: &crate::types::session_data::SessionData,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: &crate::types::session_id::SessionId,
) -> std::result::Result<Option<crate::types::session_data::SessionData>, String> {
    log::debug!("Checking if compaction needed for session {}", session_id);

    // Log details used in the check
    {
        let keep_last = app_state.config.compaction_policy.keep_last;
        log::debug!(
            "Session {}: Checking compaction. History length = {}, Keep last = {}",
            session_id,
            session_data.history.len(),
            keep_last
        );
    }

    if crate::conversation::compaction::should_compact_history::should_compact_history(
        &session_data.history,
        app_state.clone(),
    )
    .await
    {
        log::info!("Compaction needed for session {}", session_id);

        match crate::conversation::compaction::compact_history::compact_history(
            app_state.clone(),
            session_id,
        )
        .await
        {
            Ok(_) => {
                log::info!("History compacted successfully for session {}", session_id);
                // Re-fetch session data after compaction to reflect changes
                match crate::session::manager::get_session(app_state.clone(), session_id).await {
                    Some(updated_data) => std::result::Result::Ok(Some(updated_data)),
                    None => {
                        log::error!("Session {} not found after compaction attempt.", session_id);
                        // Propagate error: session disappeared
                        std::result::Result::Err(format!(
                            "Session {} disappeared after compaction.",
                            session_id
                        ))
                    }
                }
            }
            Err(e) => {
                log::error!("History compaction failed for session {}: {}", session_id, e);
                // Compaction failed, but maybe not fatal for the loop. Return Ok(None).
                std::result::Result::Ok(None)
            }
        }
    } else {
        log::debug!("No compaction needed for session {}", session_id);
        // No compaction needed, return Ok(None).
        std::result::Result::Ok(None)
    }
}


#[cfg(test)]
mod tests {
    // Tests need mocking for should_compact_history, compact_history, and get_session.
    #[tokio::test]
    async fn test_compaction_needed_and_succeeds() {
        // Setup mocks: should_compact -> true, compact_history -> Ok, get_session -> Some(updated_data)
        // Call handle_compaction
        // Assert Ok(Some(updated_data))
    }

    #[tokio::test]
    async fn test_compaction_needed_but_fails() {
        // Setup mocks: should_compact -> true, compact_history -> Err
        // Call handle_compaction
        // Assert Ok(None) (non-fatal error)
    }

     #[tokio::test]
    async fn test_compaction_needed_but_session_disappears() {
        // Setup mocks: should_compact -> true, compact_history -> Ok, get_session -> None
        // Call handle_compaction
        // Assert Err("...disappeared...")
    }

    #[tokio::test]
    async fn test_compaction_not_needed() {
        // Setup mocks: should_compact -> false
        // Call handle_compaction
        // Assert Ok(None)
    }
}