//! Provides the handler function for the 'save_context' agent tool.
//!
//! This function adds content provided in parameters to the session's context list
//! within the application state. It requires access to the mutable AppState.
//! Adheres strictly to the project's Rust coding standards.
//! Follows the "one item per file" rule.

// Required types - Used via FQN

/// Handler for the 'save_context' tool.
///
/// Adds content to the session's context list in `AppState`.
/// Expects parameters matching `crate::types::tool_parameters::ToolParameters::SaveContext`.
pub async fn handle_save_context(
    params: crate::types::tool_parameters::ToolParameters, // Updated type
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: crate::types::session_id::SessionId,
) -> Result<String, String> {
    match params {
        crate::types::tool_parameters::ToolParameters::SaveContext { content, source } => {
            // Use extracted content and source directly
            let context_entry = crate::types::context_entry::ContextEntry {
                content, // Already a String
                source,  // Already Option<String>
                timestamp: chrono::Utc::now(),
            };

            // Lock the Arc<Mutex<AppState>> to get mutable access to AppState content
            // Lock the Arc<Mutex<HashMap>> within AppState to get mutable access to sessions
            let mut sessions_guard = app_state.sessions.lock().await;

            // Find the session and add to its context
            if let Some(session_data) = sessions_guard.get_mut(&session_id) {
                session_data.context.push(context_entry);
                std::result::Result::Ok(std::string::String::from("Context saved successfully."))
            } else {
                std::result::Result::Err(std::format!("Session {} not found for saving context.", session_id))
            }
            // Locks are released when guards go out of scope
        }
         _ => std::result::Result::Err(std::string::String::from( // Handle incorrect parameter type
            "Invalid parameters provided for save_context tool.",
        )),
    }
}


#[cfg(test)]
mod tests {
    // Use fully qualified paths for items from parent/other modules.
    // Tests need update separately to use ToolParameters.

    // Helper function to create a default AppConfig for tests
    fn default_app_config() -> crate::config::app_config::AppConfig {
        // Provide sensible defaults or load from a test config if complex
        crate::config::app_config::AppConfig {
            database_url: "".to_string(), // Placeholder
            server_address: "127.0.0.1:8080".to_string(), // Placeholder
            // Add other required fields based on AppConfig definition
            evaluator_sleep_seconds: 30, // Example
            session_timeout_seconds: 300, // Example
        }
    }

    // Helper function to create a default SessionConfig for tests
    fn default_session_config() -> crate::types::session_config::SessionConfig {
        crate::types::session_config::SessionConfig {
             time_limit: std::time::Duration::from_secs(300),
             token_threshold: 100000,
             preserve_exchanges: 10,
        }
    }

    // Helper to create AppState with a specific session for testing
    async fn setup_test_app_state_with_session(session_id: crate::types::session_id::SessionId) -> actix_web::web::Data<crate::state::app_state::AppState> {
        let dummy_session_data = crate::types::session_data::SessionData { // Correct type path
            status: crate::types::session_status::SessionStatus::Pending, // Use correct enum path
            config: default_session_config(),
            history: vec![],
            context: vec![], // Start with empty context
            created_at: chrono::Utc::now(),
        };
        let mut initial_sessions = std::collections::HashMap::new();
        initial_sessions.insert(session_id, dummy_session_data);

        let app_state_content = crate::state::app_state::AppState {
             config: default_app_config(),
             sessions: std::sync::Arc::new(tokio::sync::Mutex::new(initial_sessions)), // Correct structure
             ws_connections: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())), // Add ws_connections based on AppState definition
        };
        std::sync::Arc::new(tokio::sync::Mutex::new(app_state_content))
    }

    #[tokio::test]
    async fn test_handle_save_context_success() {
        // Test successful saving of context.
        // Needs update to use ToolParameters::SaveContext
        let parameters = crate::types::tool_parameters::ToolParameters::SaveContext {
            content: "Test data".to_string(),
            source: Some("test".to_string())
        };
        let dummy_session_id = crate::types::session_id::SessionId::new_v4(); // Use correct type path for Uuid alias
        let app_state = setup_test_app_state_with_session(dummy_session_id).await;

        // Call the actual function under test
        let result = super::handle_save_context(parameters, app_state.clone(), dummy_session_id).await;

        std::assert!(result.is_ok());
        std::assert_eq!(result.unwrap(), "Context saved successfully.");

        // Verify state modification
        let state_guard = app_state.lock().await;
        let sessions_guard = state_guard.sessions.lock().await;
        let session_context = &sessions_guard[&dummy_session_id].context;
        std::assert_eq!(session_context.len(), 1);
        std::assert_eq!(session_context[0].content, "Test data");
         std::assert_eq!(session_context[0].source, Some("test".to_string()));
    }

     #[tokio::test]
    async fn test_handle_save_context_session_not_found() {
        // Test error handling when session ID is invalid.
        let parameters = crate::types::tool_parameters::ToolParameters::SaveContext {
            content: "Test data".to_string(),
            source: None
        };
        let non_existent_session_id = crate::types::session_id::SessionId::new_v4();
        // Create an AppState without the session ID
        let app_state_content = crate::state::app_state::AppState {
             config: default_app_config(),
             sessions: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())), // Empty sessions
             ws_connections: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        };
        let app_state = std::sync::Arc::new(tokio::sync::Mutex::new(app_state_content));

        let result = super::handle_save_context(parameters, app_state, non_existent_session_id).await;

        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().contains("not found for saving context"));
    }

     #[tokio::test]
    async fn test_handle_save_context_missing_content() {
        // Test error handling for missing required parameter.
        // This is now handled by the match statement's default arm.
        let parameters = crate::types::tool_parameters::ToolParameters::Search { query: "wrong type".to_string() }; // Example of wrong type
        let dummy_session_id = crate::types::session_id::SessionId::new_v4();
        let app_state = setup_test_app_state_with_session(dummy_session_id).await; // State needs the session to exist for this check

        let result = super::handle_save_context(parameters, app_state, dummy_session_id).await;

        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().contains("Invalid parameters provided for save_context tool.")); // Updated expected error
    }
}