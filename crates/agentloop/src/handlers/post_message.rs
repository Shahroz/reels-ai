//! Handles the request to post a message to an ongoing session, updating session state.
//!
//! Accepts a message payload for a specific session identified by SessionId in the path.
//! Retrieves the session from the application state, creates a new conversation entry,
//! appends it to the session history, and returns an appropriate HTTP response.
//! Adheres strictly to the one-item-per-file and FQN guidelines.
//! Expects integration with Actix-web for routing and AppState for session management.

//! Revision History
//! - 2025-04-24T12:45:12Z @AI: Initial stub implementation.
//! - 2025-04-24T14:06:24Z @AI: Implemented core logic for state interaction, conversation entry creation, history update, error handling, and added in-file tests, adhering to guidelines.

use crate::types::message::Message;

/// Handles POST requests to send a message to a session and update session state.
#[utoipa::path(
    post,
    path = "/loupe/session/{session_id}/message",
    tag = "Session",
    params(
        ("session_id" = Uuid, Path, description = "ID of the session to post the message to")
    ),
    request_body = Message,
    responses(
        (status = 200, description = "Message posted successfully"),
        (status = 404, description = "Session not found")
    ),
    tag = "Loupe"
)]
pub async fn post_message(
    session_id_path: actix_web::web::Path<crate::types::session_id::SessionId>,
    message_payload: actix_web::web::Json<crate::types::message::Message>,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
) -> actix_web::HttpResponse {
    let session_id: crate::types::session_id::SessionId = session_id_path.into_inner();
    let received_message: crate::types::message::Message = message_payload.into_inner();

    let effective_message_content = crate::utils::message_formatter::format_message_with_attachments(
        &received_message.content,
        if received_message.attachments.is_empty() { None } else { Some(&received_message.attachments) }
    );

    // Minimize lock duration by extracting and processing data quickly
    {
        let mut sessions = app_state.sessions.lock().await;

        // Retrieve mutable session data
        let session_data = match sessions.get_mut(&session_id) {
            Some(data) => data,
            None => {
                // Session not found, return 404 - drop lock immediately
                drop(sessions);
                std::println!("Session {} not found.", session_id);
                return actix_web::HttpResponse::NotFound().finish();
            }
        };

        // Check if the session is in a terminal state
        let is_terminal = matches!(
            session_data.status,
            crate::types::session_status::SessionStatus::Completed
                | crate::types::session_status::SessionStatus::Timeout
                | crate::types::session_status::SessionStatus::Error
                | crate::types::session_status::SessionStatus::AwaitingInput
                | crate::types::session_status::SessionStatus::Interrupted
        );

        if is_terminal {
            // If terminal (e.g., Completed), append message and set back to Pending to continue conversation
            // Do not reset the entire session, just reactivate it.
            log::info!("Continuing terminal session {} with new message. Setting status to Pending.", session_id);
            session_data.status = crate::types::session_status::SessionStatus::Pending; // Reactivate
            // Update the research goal to the new user message content (including attachments)
            session_data.research_goal = Some(effective_message_content.clone());
            log::info!("Updated research goal for session {} to: {}", session_id, effective_message_content);
        }

        // Create a new user conversation entry
        let new_entry = crate::types::conversation_entry::ConversationEntry {
            sender: crate::types::sender::Sender::User,
            message: effective_message_content, // Use the content with formatted attachments
            timestamp: chrono::Utc::now(), // Use FQN for Utc and now()
            tools: std::vec::Vec::new(), // No tools for a user message
            id: uuid::Uuid::new_v4(),
            parent_id: None,
            depth: 0,
            attachments: received_message.attachments.clone(), // User messages can have attachments
            tool_choice: None, // Not a tool choice
            tool_response: None, // Not a tool response
        };

        // Append the new entry to the session history
        session_data.history.push(new_entry);
        
        // Explicitly drop the lock to make it clear when it's released
    } // Lock is released here

    std::println!("Message added to session {}.", session_id); // Placeholder logging

    // Return OK response upon successful processing
    actix_web::HttpResponse::Ok().finish()
}

#[cfg(test)]
mod tests {
    // Use super::* to access the function under test within this file's scope.
    // NOTE: This is an exception to the 'no use' rule, specifically for the inner test module
    // to access the parent item. Other items still require FQNs.
    use super::post_message;

    // Placeholder mock types needed for testing the handler logic without full dependencies
    // These mirror the structure needed by SessionData and ConversationEntry
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct MockSessionConfig;
    // Removed MockSessionStatus, will use crate::types::session_status::SessionStatus directly
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct MockContextEntry;
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct MockToolChoice;

    // Mock SessionData structure for testing. Must match the expected fields used by the handler.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct MockSessionData {
        status: crate::types::session_status::SessionStatus, // Using actual SessionStatus
        config: MockSessionConfig,
        history: std::vec::Vec<crate::types::conversation_entry::ConversationEntry>,
        context: std::vec::Vec<MockContextEntry>,
        created_at: chrono::DateTime<chrono::Utc>,
        last_activity_timestamp: chrono::DateTime<chrono::Utc>,
        research_goal: Option<String>, // Added research_goal
    }

    // Mock AppState structure for testing. Must provide the `sessions` field.
    struct MockAppState {
        sessions: tokio::sync::Mutex<std::collections::HashMap<crate::types::session_id::SessionId, MockSessionData>>,
    }

    // Helper to create a mock AppState with an optional initial session
    fn create_mock_app_state(session_id: Option<crate::types::session_id::SessionId>, session_data: Option<MockSessionData>) -> MockAppState {
        let mut sessions_map = std::collections::HashMap::new();
        if let (Some(id), Some(data)) = (session_id, session_data) {
            sessions_map.insert(id, data);
        }
        MockAppState {
            sessions: tokio::sync::Mutex::new(sessions_map),
        }
    }

    // Helper to create minimal mock SessionData
    fn create_minimal_mock_session_data() -> MockSessionData {
         MockSessionData {
              status: crate::types::session_status::SessionStatus::Created, // Using actual SessionStatus variant
              config: MockSessionConfig,
              history: std::vec::Vec::new(),
              context: std::vec::Vec::new(),
              created_at: chrono::Utc::now(),
              last_activity_timestamp: chrono::Utc::now(),
              research_goal: None, // Initialize research_goal
         }
    }


    #[actix_web::test]
    async fn test_post_message_success_no_attachments() {
        // Setup: Create a mock session ID, message, and AppState with a session
        let test_session_id: crate::types::session_id::SessionId = std::string::String::from("test-session-123");
        let test_message_content = std::string::String::from("Hello, agent!");
        let message_payload_struct = crate::types::message::Message {
            role: "user".to_string(),
            content: test_message_content.clone(),
            attachments: std::vec::Vec::new(),
        };

        let initial_session_data = create_minimal_mock_session_data();
        let app_state_data_arc = std::sync::Arc::new(create_mock_app_state(Some(test_session_id.clone()), Some(initial_session_data.clone())));

        // Create Actix-web Data wrappers
        let session_id_path = actix_web::web::Path::from(test_session_id.clone());
        let message_payload_json = actix_web::web::Json(message_payload_struct);
        let app_state_actix_data = actix_web::web::Data::from(app_state_data_arc.clone());

        // Call the handler function
        let response = post_message(session_id_path, message_payload_json, app_state_actix_data).await;

        // Assertions:
        // 1. Check if the response status is OK (200)
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);

        // 2. Check if the message was added to the session history
        let sessions_guard = app_state_data_arc.sessions.lock().await;
        let updated_session_data = sessions_guard.get(&test_session_id).expect("Session should exist");

        // History should now have one entry
        assert_eq!(updated_session_data.history.len(), 1);

        // Check the content of the added entry
        let added_entry = &updated_session_data.history[0];
        assert_eq!(added_entry.sender, crate::types::sender::Sender::User);
        let expected_message = std::format!("<MAIN_TASK>\n{}\n</MAIN_TASK>", test_message_content);
        assert_eq!(added_entry.message, expected_message);
        // Note: Checking timestamp exact match is fragile, just assert it's there.
        assert!(!added_entry.timestamp.to_rfc3339().is_empty());
        assert!(added_entry.tools.is_empty());
    }

    #[actix_web::test]
    async fn test_post_message_session_not_found() {
        // Setup: Create a mock session ID that does NOT exist in AppState
        let non_existent_session_id: crate::types::session_id::SessionId = std::string::String::from("non-existent-session");
        let test_message_content = std::string::String::from("Should not be delivered");
        let message_payload_struct = crate::types::message::Message {
            role: "user".to_string(),
            content: test_message_content.clone(),
            attachments: std::vec::Vec::new(),
        };
        // Create AppState with no sessions
        let app_state_data_arc = std::sync::Arc::new(create_mock_app_state(None, None));

        // Create Actix-web Data wrappers
        let session_id_path = actix_web::web::Path::from(non_existent_session_id.clone());
        let message_payload_json = actix_web::web::Json(message_payload_struct);
        let app_state_actix_data = actix_web::web::Data::from(app_state_data_arc.clone());

        // Call the handler function
        let response = post_message(session_id_path, message_payload_json, app_state_actix_data).await;

        // Assertions:
        // 1. Check if the response status is NotFound (404)
        assert_eq!(response.status(), actix_web::http::StatusCode::NOT_FOUND);

        // 2. Check that the sessions map remains unchanged (no new session created, no history updated)
        let sessions_guard = app_state_data_arc.sessions.lock().await;
        assert!(sessions_guard.get(&non_existent_session_id).is_none());
        assert!(sessions_guard.is_empty()); // Should still be empty
    }

    #[actix_web::test]
    async fn test_post_message_awaiting_input_to_pending() {
        // Setup: Create a session initially in AwaitingInput state
        let test_session_id: crate::types::session_id::SessionId = std::string::String::from("test-session-awaiting");
        let test_message_content = std::string::String::from("Continuing the conversation");
        let mut initial_session_data = create_minimal_mock_session_data();
        initial_session_data.status = crate::types::session_status::SessionStatus::AwaitingInput;
        initial_session_data.research_goal = Some("Old goal".to_string()); // Set an old goal to see it change

        let app_state_data_arc = std::sync::Arc::new(create_mock_app_state(Some(test_session_id.clone()), Some(initial_session_data.clone())));

        let message_payload_struct = crate::types::message::Message {
            role: "user".to_string(),
            content: test_message_content.clone(),
            attachments: std::vec::Vec::new(),
        };

        // Create Actix-web Data wrappers
        let session_id_path = actix_web::web::Path::from(test_session_id.clone());
        let message_payload_json = actix_web::web::Json(message_payload_struct);
        let app_state_actix_data = actix_web::web::Data::from(app_state_data_arc.clone());

        // Call the handler function
        let response = post_message(session_id_path, message_payload_json, app_state_actix_data).await;

        // Assertions:
        // 1. Check if the response status is OK (200)
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);

        // 2. Check if the session status was updated to Pending and research goal updated
        let sessions_guard = app_state_data_arc.sessions.lock().await;
        let updated_session_data = sessions_guard.get(&test_session_id).expect("Session should exist");
        assert_eq!(updated_session_data.status, crate::types::session_status::SessionStatus::Pending);
        let expected_research_goal_content = std::format!("<MAIN_TASK>\n{}\n</MAIN_TASK>", test_message_content);
        assert_eq!(updated_session_data.research_goal, Some(expected_research_goal_content.clone()));

        // 3. Check if the message was added to the session history
        assert_eq!(updated_session_data.history.len(), 1);
        let added_entry = &updated_session_data.history[0];
        assert_eq!(added_entry.sender, crate::types::sender::Sender::User);
        assert_eq!(added_entry.message, expected_research_goal_content);
    }

    #[actix_web::test]
    async fn test_post_message_with_text_attachment() {
        let test_session_id: crate::types::session_id::SessionId = std::string::String::from("test-session-attach");
        let original_content = "Message with an attachment.";
        let attachment_content_str = "This is the text content of the attachment.";

        let text_attachment = crate::types::attachment::Attachment {
            title: Some("MyDoc.txt".to_string()),
            kind: crate::types::attachment_type::AttachmentType::Text(
                crate::types::text_attachment::TextAttachment {
                    content: attachment_content_str.to_string(),
                },
            ),
        };

        let message_with_attachment = crate::types::message::Message {
            role: "user".to_string(),
            content: original_content.to_string(),
            attachments: std::vec![text_attachment.clone()],
        };

        let mut initial_session_data = create_minimal_mock_session_data();
        initial_session_data.status = crate::types::session_status::SessionStatus::AwaitingInput; // To test research_goal update
        let app_state_data_arc = std::sync::Arc::new(create_mock_app_state(Some(test_session_id.clone()), Some(initial_session_data)));

        let session_id_path = actix_web::web::Path::from(test_session_id.clone());
        let message_payload_json = actix_web::web::Json(message_with_attachment);
        let app_state_actix_data = actix_web::web::Data::from(app_state_data_arc.clone());

        let response = post_message(session_id_path, message_payload_json, app_state_actix_data).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);

        let sessions_guard = app_state_data_arc.sessions.lock().await;
        let updated_session_data = sessions_guard.get(&test_session_id).expect("Session should exist");
        assert_eq!(updated_session_data.history.len(), 1);

        let attachments_vec = std::vec![text_attachment.clone()];
        let expected_json_attachments = serde_json::to_string_pretty(&attachments_vec)
            .expect("Test setup: Failed to serialize test attachments for expected message");
        let expected_effective_message = std::format!(
            "<ADDITIONAL_CONTEXT>\n{}\n</ADDITIONAL_CONTEXT>\n\n<MAIN_TASK>\n{}\n</MAIN_TASK>",
            expected_json_attachments,
            original_content
        );

        let added_entry = &updated_session_data.history[0];
        assert_eq!(added_entry.message, expected_effective_message);

        // Check research_goal update as status was AwaitingInput
        assert_eq!(updated_session_data.status, crate::types::session_status::SessionStatus::Pending);
        assert_eq!(updated_session_data.research_goal, Some(expected_effective_message));
    }
}
