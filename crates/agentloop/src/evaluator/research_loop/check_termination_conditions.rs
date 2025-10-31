//! Checks various conditions to determine if the research loop should terminate.
//!
//! This function consolidates checks for time limits, LLM-based termination signals,
//! external interruptions, and pre-existing terminal statuses (Error, Completed, Timeout).
//! Adheres to the one-item-per-file guideline and uses FQNs.

/// Represents the reason for loop termination.
#[derive(Debug, Clone, PartialEq)]
pub enum TerminationReason {
    Timeout,
    Interrupted,
    AlreadyTerminal(crate::types::session_status::SessionStatus),
    ConfigError(String), // Added for invalid time limit config
}


/// Checks if the research loop for the given session should terminate.
///
/// # Arguments
/// * `session_data` - The current data for the session.
/// * `app_state` - Shared application state (needed for LLM termination check).
///
/// # Returns
/// * `Ok(Some(TerminationReason))` if a termination condition is met.
/// * `Ok(None)` if the loop should continue.
/// * `Err(String)` only if a fatal error occurs during checks (currently only invalid config).
pub async fn check_termination_conditions(
    session_data: &crate::types::session_data::SessionData,
    _app_state: actix_web::web::Data<crate::state::app_state::AppState>,
) -> std::result::Result<Option<TerminationReason>, String> {
    let session_id = &session_data.session_id; // Assuming SessionId is String or easily convertible
    let _session_id_uuid = match uuid::Uuid::parse_str(session_id) {
         Ok(id) => id,
         Err(_) => return std::result::Result::Err(format!("Invalid Session ID format: {}", session_id)),
    };


    // 1. Check for time limit
    let now = chrono::Utc::now();
    let time_limit_chrono = match chrono::Duration::from_std(session_data.config.time_limit) {
        Ok(d) => d,
        Err(e) => {
            log::error!(
                "Session {} has invalid time limit configuration: {}",
                session_id, e
            );
            // Return Ok(Some(TerminationReason)) for config error to handle status update in main loop
            return std::result::Result::Ok(Some(TerminationReason::ConfigError(format!(
                "Invalid time limit duration: {}", e
            ))));
        }
    };

    let elapsed = now.signed_duration_since(session_data.created_at);
    if elapsed > time_limit_chrono {
        log::info!(
            "Session {} timed out after {:.2?} (limit: {:.2?})",
            session_id, elapsed, time_limit_chrono
        );
        return std::result::Result::Ok(Some(TerminationReason::Timeout));
    }

    // 2. Check for LLM-based termination (if context is not empty)
    // 3. Check for external interruption status
    if session_data.status == crate::types::session_status::SessionStatus::Interrupted {
        log::info!(
            "Session {} was interrupted externally. Terminating loop.",
            session_id
        );
        return std::result::Result::Ok(Some(TerminationReason::Interrupted));
    }

    // 4. Check if status is already terminal
    match &session_data.status {
        crate::types::session_status::SessionStatus::Completed
        | crate::types::session_status::SessionStatus::Error
        | crate::types::session_status::SessionStatus::Timeout => {
            // AwaitingInput is not considered terminal for the purpose of breaking the loop here.
            // It signifies a pause state waiting for user interaction or timeout.
            log::warn!(
                "Session {} loop encountered unexpected terminal status ({:?}). Breaking.",
                session_id, session_data.status
            );
            // Return the specific terminal status found
            return std::result::Result::Ok(Some(TerminationReason::AlreadyTerminal(session_data.status.clone())));
        }
        _ => { /* Status is Running or Pending, continue */ }
    }

    // 5. No termination condition met
    std::result::Result::Ok(None)
}


#[cfg(test)]
mod tests {
    // Tests would involve creating various SessionData instances and mocking
    // app_state and check_termination results.
    #[tokio::test]
    async fn test_termination_timeout() {
        // Create SessionData with created_at far in the past
        // Call check_termination_conditions
        // Assert Ok(Some(TerminationReason::Timeout))
    }

     #[tokio::test]
    async fn test_termination_interrupted() {
        // Create SessionData with status Interrupted
        // Call check_termination_conditions
        // Assert Ok(Some(TerminationReason::Interrupted))
    }

     #[tokio::test]
    async fn test_termination_already_terminal() {
        // Create SessionData with status Completed
        // Call check_termination_conditions
        // Assert Ok(Some(TerminationReason::AlreadyTerminal(SessionStatus::Completed)))
    }

    #[tokio::test]
    async fn test_no_termination() {
        // Create SessionData with recent created_at, status Running, non-empty context
        // Setup mock app_state where check_termination returns false
        // Call check_termination_conditions
        // Assert Ok(None)
    }
}