//! Manages research sessions within the application state.
//!
//! Provides asynchronous functions to create, retrieve, and modify session data
//! stored within the central `AppState`, ensuring thread-safe access
//! via `tokio::sync::Mutex`. This includes managing status,
//! conversation history, and collected context.
//! Adheres to the project's Rust coding guidelines.

//! Revision History
//! - 2025-04-24T15:58:49Z @AI: Update tests to match new SessionStatus::Running variant.
//! - 2025-04-24T14:39:42Z @AI: Fix type resolution errors (E0433, E0573).
//! - 2025-04-24T13:45:06Z @AI: Refactor functions for async locking (`tokio::sync::Mutex`).
//! - 2025-04-24T13:39:54Z @AI: Update paths from models::session_data to types::session_data.

// Note: Explicitly using fully qualified paths as per guidelines.

// Function to create a new session asynchronously
pub async fn create_session(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    config: crate::types::session_config::SessionConfig, // Correct path
) -> crate::types::session_id::SessionId { // Correct path: Use type alias from module
    let session_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();

    // Initialize research_goal from the initial instruction in the session config
    let research_goal = config.initial_instruction.clone();

    // Combine internal and external tool definitions for this session
    let session_data = crate::types::session_data::SessionData { // Correct path
        session_id: session_id.to_string(),
        status: crate::types::session_status::SessionStatus::Pending,
        config, // Session specific config
        history: std::vec::Vec::new(),
        context: std::vec::Vec::new(),
        research_goal,
        created_at: now,
        last_activity_timestamp: now,
        messages: std::vec::Vec::new(),
        system_message: std::option::Option::None
    };
    
    // Lock the sessions map asynchronously and insert the new session.
    let mut sessions_guard = app_state.sessions.lock().await;
    sessions_guard.insert(session_id, session_data);
    session_id
}

// Function to get a clone of session data asynchronously
// Returns an Option<SessionData> clone, avoiding problematic mutable references across awaits.
pub async fn get_session(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: &crate::types::session_id::SessionId, // Correct path
) -> std::option::Option<crate::types::session_data::SessionData> { // Correct path
    let sessions_guard = app_state.sessions.lock().await;
    sessions_guard.get(session_id).cloned()
    // Lock released when guard goes out of scope.
}


// Function to update the status of a session asynchronously
pub async fn update_status(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: &crate::types::session_id::SessionId, // Correct path
    new_status: crate::types::session_status::SessionStatus, // Correct path
) -> std::result::Result<(), std::string::String> {
    let mut sessions_guard = app_state.sessions.lock().await;
    if let std::option::Option::Some(session) = sessions_guard.get_mut(session_id) {
        session.status = new_status;
        session.last_activity_timestamp = chrono::Utc::now(); // Update activity timestamp on status change
        std::result::Result::Ok(())
    } else {
        std::result::Result::Err(std::format!("Session not found: {}", session_id))
    }
    // Lock released when guard goes out of scope.
}

// Function to add a conversation entry to a session's history asynchronously
pub async fn add_conversation_entry(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: &crate::types::session_id::SessionId, // Correct path
    entry: crate::types::conversation_entry::ConversationEntry, // Correct path
) -> std::result::Result<(), std::string::String> {
    let mut sessions_guard = app_state.sessions.lock().await;
    if let std::option::Option::Some(session) = sessions_guard.get_mut(session_id) {
        session.history.push(entry);
        session.last_activity_timestamp = chrono::Utc::now(); // Update activity timestamp
        std::result::Result::Ok(())
    } else {
        std::result::Result::Err(std::format!("Session not found: {}", session_id))
    }
    // Lock released when guard goes out of scope.
}

// Function to add a context entry to a session asynchronously
pub async fn add_context_entry(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: &crate::types::session_id::SessionId, // Correct path
    entry: crate::types::context_entry::ContextEntry, // Correct path
) -> std::result::Result<(), std::string::String> {
    let mut sessions_guard = app_state.sessions.lock().await;
    if let std::option::Option::Some(session) = sessions_guard.get_mut(session_id) {
        session.context.push(entry);
        session.last_activity_timestamp = chrono::Utc::now(); // Update activity timestamp
        std::result::Result::Ok(())
    } else {
        std::result::Result::Err(std::format!("Session not found: {}", session_id))
    }
    // Lock released when guard goes out of scope.
}


#[cfg(test)]
mod tests {
    // Use fully qualified paths for imports and types.
    // Tests are now async using tokio::test.

    // Helper to create a default AppState for async tests.
    fn create_async_test_app_state() -> actix_web::web::Data<crate::state::app_state::AppState> {
         let config = crate::config::app_config::AppConfig::default();
         let app_state = crate::state::app_state::AppState::new(config, None, None);
         actix_web::web::Data::new(app_state)
    }

    // Helper for default SessionConfig, assuming it has a Default impl or simple constructor.
    fn default_session_config() -> crate::types::session_config::SessionConfig { // Correct path
        // Assuming SessionConfig can be defaulted or constructed easily.
        // Replace with actual construction if needed.
        crate::types::session_config::SessionConfig { // Correct path
             time_limit: std::time::Duration::from_secs(300), // Example
             token_threshold: 1000, // Example
             preserve_exchanges: 5, // Example
        }
    }


    #[tokio::test]
    async fn test_create_and_get_session_async() {
        let app_state = create_async_test_app_state();
        let config = default_session_config();

        // Create session asynchronously.
        let session_id = super::create_session(app_state.clone(), config.clone()).await;

        // Verify session count within the lock.
        let sessions_guard = app_state.sessions.lock().await;
        std::assert_eq!(sessions_guard.len(), 1);
        drop(sessions_guard); // Explicitly drop guard before next await if needed, though usually implicit.

        // Get session clone asynchronously.
        let session_opt = super::get_session(app_state, &session_id).await;
        std::assert!(session_opt.is_some());
        let session = session_opt.unwrap();

        // Assertions on the cloned session data.
        std::assert_eq!(session.status, crate::types::session_status::SessionStatus::Pending); // Correct path
        // Compare relevant fields if SessionConfig doesn't impl PartialEq
        std::assert_eq!(session.config.time_limit, config.time_limit);
        std::assert_eq!(session.config.token_threshold, config.token_threshold);
        std::assert!(session.history.is_empty());
        std::assert!(session.context.is_empty());
    }

    #[tokio::test]
    async fn test_update_status_async() {
        let app_state = create_async_test_app_state();
        let config = default_session_config();
        let session_id = super::create_session(uuid::Uuid::new_v4(), app_state.clone(), config.clone(), None).await;

        // Update status asynchronously using the new Running variant structure.
        let new_status = crate::types::session_status::SessionStatus::Running { progress: std::option::Option::Some(std::string::String::from("50%")) };
        let update_result = super::update_status(app_state.clone(), &session_id, new_status.clone()).await;
        std::assert!(update_result.is_ok());

        // Verify update by getting session clone.
        let session = super::get_session(app_state.clone(), &session_id).await.unwrap();
        std::assert_eq!(session.status, new_status);

        // Test updating non-existent session.
        let fake_id = uuid::Uuid::new_v4();
        // Use a different status for the failure case test.
        let update_result_fail = super::update_status(app_state.clone(), &fake_id, crate::types::session_status::SessionStatus::Error).await;
        std::assert!(update_result_fail.is_err());
        std::assert_eq!(update_result_fail.unwrap_err(), std::format!("Session not found: {}", fake_id));
    }

     #[tokio::test]
     async fn test_add_conversation_entry_async() {
         let app_state = create_async_test_app_state();
         let config = default_session_config();
         let session_id = super::create_session(uuid::Uuid::new_v4(), app_state.clone(), config.clone(), None).await;

         let entry = crate::types::conversation_entry::ConversationEntry { // Correct path
             // Use fully qualified path for Sender enum
             sender: crate::types::sender::Sender::User,
             message: std::string::String::from("Hello"),
             timestamp: chrono::Utc::now(),
             tools: std::vec::Vec::new(),
             id: uuid::Uuid::new_v4(),
             parent_id: None,
             depth: 0,
             attachments: std::vec::Vec::new(),
             tool_choice: None,
             tool_response: None,
         };

         // Add entry asynchronously.
         let add_result = super::add_conversation_entry(app_state.clone(), &session_id, entry.clone()).await;
         std::assert!(add_result.is_ok());

         // Verify by getting session clone.
         let session = super::get_session(app_state.clone(), &session_id).await.unwrap();
         std::assert_eq!(session.history.len(), 1);
         // Add comparison for entry fields if ConversationEntry impl PartialEq
         // std::assert_eq!(session.history[0], entry);

         // Test adding to non-existent session.
         let fake_id = uuid::Uuid::new_v4();
         let add_result_fail = super::add_conversation_entry(app_state.clone(), &fake_id, entry).await;
         std::assert!(add_result_fail.is_err());
         std::assert_eq!(add_result_fail.unwrap_err(), std::format!("Session not found: {}", fake_id));
     }

     #[tokio::test]
     async fn test_add_context_entry_async() {
         let app_state = create_async_test_app_state();
         let config = default_session_config();
         let session_id = super::create_session(uuid::Uuid::new_v4(), app_state.clone(), config.clone(), None).await;

         let entry = crate::types::context_entry::ContextEntry { // Correct path
             content: std::string::String::from("Some context"),
             source: std::option::Option::Some(std::string::String::from("test.txt")),
             timestamp: chrono::Utc::now(),
         };

         // Add entry asynchronously.
         let add_result = super::add_context_entry(app_state.clone(), &session_id, entry.clone()).await;
         std::assert!(add_result.is_ok());

         // Verify by getting session clone.
         let session = super::get_session(app_state.clone(), &session_id).await.unwrap();
         std::assert_eq!(session.context.len(), 1);
         // Add comparison for entry fields if ContextEntry impl PartialEq
         // std::assert_eq!(session.context[0], entry);

         // Test adding to non-existent session.
         let fake_id = uuid::Uuid::new_v4();
         let add_result_fail = super::add_context_entry(app_state.clone(), &fake_id, entry).await;
         std::assert!(add_result_fail.is_err());
         std::assert_eq!(add_result_fail.unwrap_err(), std::format!("Session not found: {}", fake_id));
     }
}
