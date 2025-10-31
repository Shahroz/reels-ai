//! Implements the actual logic for the Narrativ save_context tool.
//!
//! This function takes content and an optional source, and saves it to the
//! agent's session context managed by AgentLoop.
//! Adheres to Narrativ and AgentLoop coding standards.

// Note: Using fully qualified paths as per guidelines where applicable.

/// Saves a piece of text content to the agent's context.
///
/// # Arguments
///
/// * `params` - `crate::agent_tools::tool_params::SaveContextParams` containing the content and source.
/// * `agentloop_state` - Shared application state from AgentLoop (`actix_web::web::Data<agentloop::state::app_state::AppState>`).
/// * `session_id` - The ID of the current AgentLoop session (`agentloop::types::session_id::SessionId`).
///
/// # Returns
///
/// A `Result` containing a tuple of
/// `(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse)`
/// on success, or an error `String` on failure (e.g., session not found).
pub async fn handle_narrativ_save_context(
    params: crate::agent_tools::tool_params::save_context_params::SaveContextParams,
    agentloop_state: actix_web::web::Data<agentloop::state::app_state::AppState>,
    session_id: agentloop::types::session_id::SessionId,
) -> Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    String,
> {
    let tool_name = "narrativ_save_context".to_string();

    let context_entry = agentloop::types::context_entry::ContextEntry {
        content: params.content,
        source: params.source,
        timestamp: chrono::Utc::now(),
    };

    let mut sessions_guard = agentloop_state.sessions.lock().await;

    if let Some(session_data) = sessions_guard.get_mut(&session_id) {
        session_data.context.push(context_entry);
        Ok((
            agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.clone(),
                response: serde_json::json!({"status": "success", "message": "Context saved successfully."}),
            },
            agentloop::types::user_tool_response::UserToolResponse {
                tool_name,
                summary: "Content saved to context.".to_string(),
                data: Some(
                    serde_json::json!({"status": "success", "message": "Context saved successfully."}),
                ),
                icon: Some("💾".to_string()),
            },
        ))
    } else {
        Err(format!(
            "Session {session_id} not found for saving context via Narrativ tool."
        ))
    }
}

#[cfg(test)]
mod tests {
    use agentloop::types::session_config;
    // Basic test structure for save_context.
    #[tokio::test]
    async fn test_handle_narrativ_save_context_session_found() {
        let session_id = uuid::Uuid::new_v4();
        let app_config = agentloop::config::app_config::AppConfig::default();
        let agentloop_state_inner =
            agentloop::state::app_state::AppState::new(app_config, None, None);
        // Add a session
        {
            let mut sessions = agentloop_state_inner.sessions.lock().await;
            // Assume SessionConfig holds timeout and max_context_entries
            let session_data_config = session_config::SessionConfig {
                time_limit: std::time::Duration::from_secs(3600), // Replaced timeout_seconds, assumed Option<Duration>
                // max_context_entries removed as it's not a valid field
                // Fill other SessionConfig fields with defaults
                ..session_config::SessionConfig::default()
            };
            let session_data_instance = agentloop::types::session_data::SessionData {
                user_id: uuid::Uuid::new_v4(),
                organization_id: None,
                session_id: session_id.to_string(), // Corrected field name
                research_goal: None,                // Default for Option<String>
                status: Default::default(),         // Assumes SessionStatus implements Default
                config: session_data_config,        // Use the constructed config
                history: std::vec::Vec::new(),      // Default for Vec
                context: std::vec::Vec::new(),      // Default for Vec, critical for this test logic
                messages: std::vec::Vec::new(),     // Added missing field
                system_message: None,               // Added missing field, assumed Option<String>
                created_at: chrono::Utc::now(),     // Common timestamp field
                last_activity_timestamp: chrono::Utc::now(), // Common timestamp field
            };
            sessions.insert(session_id, session_data_instance);
        }
        let agentloop_state = actix_web::web::Data::new(agentloop_state_inner);

        let params = crate::agent_tools::tool_params::save_context_params::SaveContextParams {
            content: "Test content".to_string(),
            source: Some("test_source".to_string()),
        };

        let result =
            super::handle_narrativ_save_context(params, agentloop_state.clone(), session_id).await;
        assert!(result.is_ok());
        if let Ok((full_res, user_res)) = result {
            assert_eq!(full_res.tool_name, "narrativ_save_context");
            assert_eq!(
                full_res.response.get("status"),
                Some(&serde_json::json!("success"))
            );
            assert_eq!(user_res.tool_name, "narrativ_save_context");
            assert_eq!(user_res.summary, "Content saved to context.");
        }

        // Verify context was added
        let sessions_guard = agentloop_state.sessions.lock().await;
        let session_data = sessions_guard.get(&session_id).unwrap();
        assert_eq!(session_data.context.len(), 1);
        assert_eq!(session_data.context[0].content, "Test content");
    }

    #[tokio::test]
    async fn test_handle_narrativ_save_context_session_not_found() {
        let session_id = uuid::Uuid::new_v4(); // Non-existent session
        let app_config = agentloop::config::app_config::AppConfig::default();
        let agentloop_state_inner =
            agentloop::state::app_state::AppState::new(app_config, None, None);
        let agentloop_state = actix_web::web::Data::new(agentloop_state_inner);

        let params = crate::agent_tools::tool_params::save_context_params::SaveContextParams {
            content: "Test content".to_string(),
            source: Some("test_source".to_string()),
        };

        let result = super::handle_narrativ_save_context(params, agentloop_state, session_id).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!(
                "Session {} not found for saving context via Narrativ tool.",
                session_id
            )
        );
    }
}
