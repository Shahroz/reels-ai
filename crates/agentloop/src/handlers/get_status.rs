//! Handles the request to get the status of a specific session.
//!
//! Retrieves the session status and calculates the remaining time until timeout
//! based on the SessionId provided in the path and the session data stored in AppState.
//! Returns the status information or a 404 if the session is not found.
//! Adheres to the one-item-per-file and FQN guidelines.

//! Revision History
//! - 2025-04-24T17:16:11Z @AI: Fix E0597 by ensuring AppState lock outlives sessions lock guard usage.
//! - 2025-04-24T17:13:06Z @AI: Fix E0716 temporary value dropped while borrowed by separating lock scopes.
//! - 2025-04-24T14:56:58Z @AI: Fix type errors E0308 on lines 36 and 51.
//! - 2025-04-24T14:38:54Z @AI: Fix type resolution errors for SessionId and StatusResponse. Use re-exported types.
//! - 2025-04-24T14:11:08Z @AI: Implemented full logic based on AppState.
//! - 2025-04-24T12:45:12Z @AI: Initial stub implementation.

use crate::types::status_response::StatusResponse;

/// Handles GET requests to query session status.
/// Expects SessionId in the path and AppState via web::Data.
#[utoipa::path(
    get,
    path = "/loupe/session/{session_id}/status",
    tag = "Session",
    params(
        ("session_id" = Uuid, Path, description = "ID of the session to query")
    ),
    responses(
        (status = 200, description = "Session status retrieved successfully", body = StatusResponse),
        (status = 404, description = "Session not found")
    ),
    tag = "Loupe"
)]
pub async fn get_status(
    session_id_path: actix_web::web::Path<crate::types::session_id::SessionId>, // Corrected: Use re-exported SessionId type
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
) -> impl actix_web::Responder {
    let session_id = session_id_path.into_inner();
    // Consider using a proper logging framework instead of println!
    std::println!("Received get_status request for session: {}", session_id);

    // Acquire outer lock first to get config value needed later.
    let timeout_duration = {
        std::time::Duration::from_secs(app_state.config.session_timeout_seconds)
        // state_config_lock dropped here
    };

    // Lock AppState again to access sessions. Hold this lock until sessions_guard is no longer needed.
    let sessions_guard = app_state.sessions.lock().await; // Lock inner sessions

    // Perform the lookup using the sessions_guard while the state_sessions_lock is held.
    match sessions_guard.get(&session_id) {
        std::option::Option::Some(session_data) => {
            // Session found, extract data and calculate remaining time
            let status: crate::types::session_status::SessionStatus = session_data.status.clone();
            let last_activity = session_data.last_activity_timestamp;
            // timeout_duration is already fetched

            // Convert std::time::Duration to chrono::Duration
            let chrono_timeout_duration = chrono::Duration::from_std(timeout_duration)
                .expect("Failed to convert std::time::Duration to chrono::Duration");

            let expiration_time = last_activity + chrono_timeout_duration;
            let now = chrono::Utc::now();

            // Calculate remaining time
            let time_remaining: std::option::Option<std::time::Duration> = if expiration_time > now {
                 expiration_time.signed_duration_since(now).to_std().ok()
            } else {
                 std::option::Option::None
            };

            // Construct the response data *before* dropping locks
            let response_data = crate::types::status_response::StatusResponse {
                session_id: session_id.to_string(),
                status,
                time_remaining,
            };

            // Explicitly drop guards to release locks before returning the response.
            // This isn't strictly necessary for correctness here as they would drop
            // at the end of the scope anyway, but it makes the lock duration explicit.
            drop(sessions_guard);

            actix_web::HttpResponse::Ok().json(response_data)
        }
        std::option::Option::None => {
            // Session not found. Drop guards before returning.
            drop(sessions_guard);

            std::println!("Session not found: {}", session_id);
            actix_web::HttpResponse::NotFound().finish()
        }
    }
    // Locks are guaranteed to be released by this point.
}

#[cfg(test)]
mod tests {
    // Basic tests could be added here, potentially requiring mocking frameworks
    // or a test AppState setup. For now, we ensure the code compiles.
    #[test]
    fn placeholder_test() {
        // This test doesn't validate runtime behavior but ensures basic syntax.
        assert!(true);
    }
}