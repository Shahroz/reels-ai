//! Handler for loading a session state from a request payload.
//!
//! Provides an endpoint to recreate a session based on previously saved state,
//! generating a new session ID for the loaded session.
//! Adheres to project guidelines including FQN and documentation.

// Note: Using fully qualified paths as per guidelines.

/// Loads a session state from the request body.
///
/// Path: POST /session/load
///
/// Accepts a `LoadSessionRequest` JSON object in the request body.
/// Creates a new session based on the provided state, assigns a new unique
/// session ID, and stores it in the application state. Returns the new
/// session ID upon successful creation.
#[utoipa::path(
    post,
    path = "/session/load",
    request_body = crate::types::load_session_request::LoadSessionRequest,
    responses(
        (status = 201, description = "Session loaded successfully, returns new session ID", body = inline(String)),
        (status = 400, description = "Invalid request body format")
        // Consider adding 500 for internal errors if applicable
    ),
    tag = "Session Management"
)]
pub async fn load_session_state(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    load_request: actix_web::web::Json<crate::types::load_session_request::LoadSessionRequest>,
) -> actix_web::HttpResponse {
    let request_data = load_request.into_inner();
    let new_session_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();

    // Construct the full SessionData for storage
    let new_session_data = crate::types::session_data::SessionData {
        organization_id: request_data.organization_id,
        user_id: request_data.user_id,
        session_id: new_session_id.to_string(),
        status: request_data.status,
        config: request_data.config,
        history: request_data.history,
        context: request_data.context,
        research_goal: request_data.research_goal,
        created_at: now, // Set new creation time
        last_activity_timestamp: now, // Set new activity time
        system_message: request_data.system_message,
        messages: request_data.messages,
    };

    // Lock the sessions map and insert the new session data
    let mut sessions_guard = app_state.sessions.lock().await;
    sessions_guard.insert(new_session_id, new_session_data);
    // MutexGuard is dropped here, releasing the lock.

    // Return the new session ID
    actix_web::HttpResponse::Created()
        .json(serde_json::json!({ "session_id": new_session_id.to_string() }))
}

#[cfg(test)]
mod tests {
    // Using fully qualified paths.

    // Helper to create AppState for tests.
    fn create_test_app_state() -> actix_web::web::Data<crate::state::app_state::AppState> {
        actix_web::web::Data::new(crate::state::app_state::AppState::new(
            crate::config::app_config::AppConfig::default(), // Assuming default is sufficient
            None, // tool_schemas
            None, // tool_handler
        ))
    }

    // Helper to create a default SessionConfig.
    fn default_session_config() -> crate::types::session_config::SessionConfig {
        crate::types::session_config::SessionConfig {
            time_limit: std::time::Duration::from_secs(600), // Different value for distinction
            token_threshold: 2000,
            preserve_exchanges: 10,
            initial_instruction: std::option::Option::Some("Loaded instruction".to_string()),
            compaction_policy: crate::types::compaction_policy::CompactionPolicy::default(),
            evaluation_policy: crate::types::evaluation_policy::EvaluationPolicy::default(),
        }
    }

    #[tokio::test]
    async fn test_load_session_state_success() {
        let app_state = create_test_app_state();

        let load_request_payload = crate::types::load_session_request::LoadSessionRequest {
            user_id: uuid::Uuid::new_v4(),
            organization_id: Some(uuid::Uuid::new_v4()),
            status: crate::types::session_status::SessionStatus::Running { progress: Some("Loaded state".to_string()) },
            config: default_session_config(),
            history: std::vec![crate::types::conversation_entry::ConversationEntry {
                id: uuid::Uuid::new_v4(),
                parent_id: None,
                depth: 0,
                sender: crate::types::sender::Sender::User,
                message: "Loaded message".to_string(),
                timestamp: chrono::Utc::now(),
                tools: std::vec::Vec::new(),
                attachments: std::vec::Vec::new(),
                tool_choice: None,
                tool_response: None,
            }],
            context: std::vec![crate::types::context_entry::ContextEntry {
                content: "Loaded context".to_string(),
                source: Some("loaded_source.txt".to_string()),
                timestamp: chrono::Utc::now(),
            }],
            research_goal: Some("Loaded goal".to_string()),
            system_message: Some("Loaded system prompt".to_string()),
            messages: std::vec![crate::types::message::Message::user("Loaded raw message".to_string())],
        };

        let json_payload = actix_web::web::Json(load_request_payload.clone());
        let resp = super::load_session_state(app_state.clone(), json_payload).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

        // Extract the new session ID from the response body
        let body = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let new_session_id_str = response_json["session_id"].as_str().unwrap();
        let new_session_id = uuid::Uuid::parse_str(new_session_id_str).unwrap();

        // Verify the session was actually created in the state
        let sessions_guard = app_state.sessions.lock().await;
        assert!(sessions_guard.contains_key(&new_session_id));
        let loaded_session_data = sessions_guard.get(&new_session_id).unwrap();

        // Assertions on the loaded data
        assert_eq!(loaded_session_data.session_id, new_session_id_str);
        // Compare relevant fields (assuming SessionStatus and SessionConfig implement PartialEq or compare manually)
        // assert_eq!(loaded_session_data.status, load_request_payload.status); // Status might not impl PartialEq easily
        match &loaded_session_data.status {
             crate::types::session_status::SessionStatus::Running { progress } => assert_eq!(progress.as_deref(), Some("Loaded state")),
             _ => panic!("Incorrect status loaded"),
        }
        // Simple field check for config
        assert_eq!(loaded_session_data.config.time_limit, load_request_payload.config.time_limit);
        assert_eq!(loaded_session_data.history.len(), 1);
        assert_eq!(loaded_session_data.history[0].message, "Loaded message");
        assert_eq!(loaded_session_data.context.len(), 1);
        assert_eq!(loaded_session_data.context[0].content, "Loaded context");
        assert_eq!(loaded_session_data.research_goal, Some("Loaded goal".to_string()));
        assert_eq!(loaded_session_data.system_message, Some("Loaded system prompt".to_string()));
        assert_eq!(loaded_session_data.messages.len(), 1);
        assert_eq!(loaded_session_data.messages[0].content, "Loaded raw message");

        // Check timestamps were set recently (allow some tolerance)
        let now = chrono::Utc::now();
        assert!((now - loaded_session_data.created_at).num_seconds() < 5);
        assert!((now - loaded_session_data.last_activity_timestamp).num_seconds() < 5);
    }

    // Test for bad request (invalid JSON) would typically be handled by Actix-web framework
    // before the handler is called, resulting in a 400 Bad Request response automatically.
    // Explicitly testing requires crafting a request with malformed JSON.
}