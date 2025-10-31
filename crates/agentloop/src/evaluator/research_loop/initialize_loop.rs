//! Handles the initial setup for the research loop.
//!
//! Specifically, this function updates the session status to 'Running'.
//! It is the first step executed when the research loop begins.
//! Adheres to the one-item-per-file guideline and uses FQNs.

/// Updates the session status to Running.
///
/// Logs the status update or any errors encountered during the update.
///
/// # Arguments
/// * `app_state` - Shared application state.
/// * `session_id` - The ID of the session being initialized.
///
/// # Returns
/// * `Ok(())` if the status was updated successfully.
/// * `Err(String)` if updating the status failed.
pub async fn initialize_loop(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: &crate::types::session_id::SessionId,
) -> std::result::Result<(), String> {
    log::info!("Initializing research loop for session {}", session_id);
    match crate::session::manager::update_status(
        app_state.clone(),
        session_id,
        crate::types::session_status::SessionStatus::Running { progress: None },
    )
    .await
    {
        Ok(_) => {
            log::info!("Session {} status updated to Running", session_id);
            std::result::Result::Ok(())
        }
        Err(e) => {
            log::error!(
                "Failed to update session {} status to Running: {}",
                session_id,
                e
            );
            std::result::Result::Err(format!(
                "Failed to set session status to Running: {}",
                e
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests would require mocking AppState and session::manager::update_status.
    #[tokio::test]
    async fn test_initialization_success() {
        // Setup mock AppState and expected calls
        // ...
        // Call initialize_loop
        // ...
        // Assert status was updated (mock verification)
        // ...
    }

    #[tokio::test]
    async fn test_initialization_failure() {
        // Setup mock AppState where update_status fails
        // ...
        // Call initialize_loop
        // ...
        // Assert Err result is returned
        // ...
    }
}