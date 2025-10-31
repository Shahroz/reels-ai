//! Handles the request to terminate a specific session.
//!
//! Terminates the session identified by SessionId provided in the path.
//! Removes the session state from AppState and notifies any connected
//! WebSocket clients that the session has been terminated.
//! Adheres to the one-item-per-file and FQN guidelines.

//! Revision History
//! - 2025-04-24T14:04:47Z @AI: Implement full termination logic and add tests.
//! - 2025-04-24T12:45:12Z @AI: Initial stub implementation.

/// Handles POST requests to terminate a session.
///
/// Extracts the SessionId from the path and the AppState from application data.
/// Removes the session from the shared state maps and notifies WebSocket clients.
#[utoipa::path(
    post, // Or delete, depending on final API design choice
    path = "/loupe/session/{session_id}/terminate",
    tag = "Session",
    params(
        ("session_id" = Uuid, Path, description = "ID of the session to terminate")
    ),
    responses(
        (status = 200, description = "Session terminated (or did not exist)")
        // No specific error response for not found, as it returns OK anyway.
    ),
    tag = "Loupe"
)]
pub async fn terminate_session(
    session_id_path: actix_web::web::Path<crate::types::session_id::SessionId>,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
) -> impl actix_web::Responder {
    let session_id = session_id_path.into_inner();
    std::println!("Received terminate_session request for session: {}", session_id); // Placeholder logging

    // Remove session state
    let mut sessions = app_state.sessions.lock().await;
    let removed_session_data = sessions.remove(&session_id);
    drop(sessions); // Release lock promptly

    if removed_session_data.is_some() {
        std::println!("Removed session {} state.", session_id);
    } else {
        std::println!("Session {} not found in state for termination.", session_id);
        // Decide if not found should be an error response, e.g., NotFound
        // For now, we proceed to check WS connections anyway.
    }

    // Remove and notify WebSocket connections
    let mut ws_connections = app_state.ws_connections.lock().await;
    if let Some(recipients) = ws_connections.remove(&session_id) {
        std::println!("Removed {} WebSocket connections for session {}.", recipients.len(), session_id);
        // Notify clients
        let termination_event = crate::types::ws_event::WebsocketEvent::SessionTerminated {
            session_id: session_id.to_string().clone(), // Clone SessionId for the event
            reason: Some(String::from("Session terminated by request.")), // Provide a reason
        };
        for recipient in recipients {
            // Send message, ignore errors (client might have disconnected)
            recipient.do_send(termination_event.clone());
        }
    } else {
         std::println!("No WebSocket connections found for session {}.", session_id);
    }
    drop(ws_connections); // Release lock

    // Return OK regardless of whether session/connections were found,
    // as the goal is to ensure the session is gone.
    actix_web::HttpResponse::Ok().finish()
}


#[cfg(test)]
mod tests {
    // Use fully qualified paths as per guidelines.

    // Helper to create a basic AppState for testing
    fn create_test_app_state() -> crate::state::app_state::AppState {
        let dummy_config = crate::config::app_config::AppConfig {
            database_url: String::from("dummy"),
            server_address: String::from("dummy"),
            evaluator_sleep_seconds: 60,
            session_timeout_seconds: 3600,
        };
        crate::state::app_state::AppState::new(dummy_config)
    }

    // Helper to create a session ID
    fn create_test_session_id(id: &str) -> crate::types::session_id::SessionId {
        // Assuming SessionId can be created from a String or similar
        // Adjust this based on the actual definition of SessionId
        crate::types::session_id::SessionId::new(uuid::Uuid::parse_str(id).unwrap_or_else(|_| uuid::Uuid::new_v4())) // Assuming SessionId is Uuid and providing fallback
    }

     // Helper to create dummy session data
    fn create_dummy_session_data() -> crate::types::session_data::SessionData {
        // Assuming SessionData has a simple constructor or default state
        // Adjust based on actual SessionData definition
        crate::types::session_data::SessionData {
             id: create_test_session_id("dummy-session-for-data"), // Uses corrected helper
             status: crate::types::session_status::SessionStatus::Pending, // Example status
             // Add other necessary fields with default/dummy values
             // research_goal: String::from("dummy goal"),
             // history: Vec::new(),
             // created_at: std::time::SystemTime::now(),
             // updated_at: std::time::SystemTime::now(),
        }
    }


    #[actix_rt::test]
    async fn test_terminate_removes_session_state() {
        let app_state = create_test_app_state();
        let session_id = create_test_session_id("test-session-1");
        let session_id_clone = session_id.clone(); // Clone for insertion

        // Add a session to the state
        {
    let app_state_locked = app_state.lock().await;
    let mut sessions = app_state_locked.sessions.lock().await;
            sessions.insert(session_id_clone, create_dummy_session_data());
        }

        // Verify session exists before termination
        {
            let sessions = app_state.lock().await.sessions.lock().await;
            std::assert!(sessions.contains_key(&session_id));
        }

        // Create request parts
        let path = actix_web::web::Path::from(session_id.clone());
        let data = actix_web::web::Data::new(app_state.clone()); // Clone AppState for Data

        // Call the handler
        let _resp = super::terminate_session(path, data).await;

        // Verify session is removed after termination
        {
            let sessions = app_state.lock().await.sessions.lock().await;
            std::assert!(!sessions.contains_key(&session_id));
        }
    }

    #[actix_rt::test]
    async fn test_terminate_removes_ws_connections() {
        let app_state = create_test_app_state();
        let session_id = create_test_session_id("test-session-ws");

        // Mock recipient (requires actix Actor setup or a simpler mock)
        // For simplicity, we'll just insert an empty Vec and check removal.
        // Testing the `do_send` requires more involved mocking.
        {
    let app_state_locked = app_state.lock().await; // Re-lock as the previous guard went out of scope
    let mut ws_connections = app_state_locked.ws_connections.lock().await;
            // Insert an empty Vec to represent connections exist
            ws_connections.insert(session_id.clone(), std::vec::Vec::new());
        }

        // Verify connections exist before termination
        {
            let ws_connections = app_state.lock().await.ws_connections.lock().await;
            std::assert!(ws_connections.contains_key(&session_id));
        }

        // Create request parts
        let path = actix_web::web::Path::from(session_id.clone());
        let data = actix_web::web::Data::new(app_state.clone());

        // Call the handler
        let _resp = super::terminate_session(path, data).await;

        // Verify connections are removed after termination
        {
            let ws_connections = app_state.lock().await.ws_connections.lock().await;
            std::assert!(!ws_connections.contains_key(&session_id));
        }
    }

     #[actix_rt::test]
    async fn test_terminate_non_existent_session() {
        // Test that terminating a session that doesn't exist completes without error.
        let app_state = create_test_app_state();
        let session_id = create_test_session_id("non-existent-session");

        // Verify session does not exist initially
        {
            let sessions = app_state.lock().await.sessions.lock().await;
            std::assert!(!sessions.contains_key(&session_id));
            let ws_connections = app_state.lock().await.ws_connections.lock().await;
             std::assert!(!ws_connections.contains_key(&session_id));
        }

        // Create request parts
        let path = actix_web::web::Path::from(session_id.clone());
        let data = actix_web::web::Data::new(app_state.clone());

        // Call the handler
        let resp = super::terminate_session(path, data).await;

        // Verify response is OK
        // Need to resolve the responder to check status code
        let http_resp = resp.respond_to(&actix_web::test::TestRequest::default().to_http_request());
        std::assert_eq!(http_resp.status(), actix_web::http::StatusCode::OK);


        // Verify state remains empty
         {
            let sessions = app_state.lock().await.sessions.lock().await;
            std::assert!(sessions.is_empty()); // Should still be empty
            let ws_connections = app_state.lock().await.ws_connections.lock().await;
             std::assert!(ws_connections.is_empty()); // Should still be empty
        }
    }

    // Note: Testing the actual sending of WebSocket messages requires
    // setting up mock Actors and Recipients, which is more complex.
    // The current tests verify the state removal logic.
}