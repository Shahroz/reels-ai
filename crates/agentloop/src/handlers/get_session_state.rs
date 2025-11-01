//! Handler for retrieving the full state of a session.
//!
//! Provides an endpoint to fetch the serialized `SessionData` for a given
//! session ID, allowing clients to save the session state.
//! Adheres to project guidelines including FQN and documentation.

// Note: Using fully qualified paths as per guidelines.

/// Retrieves the full state of a specific session.
///
/// Path: GET /session/{session_id}/state
///
/// Returns the complete `SessionData` object for the specified session ID
/// as a JSON response. Returns 404 if the session is not found.
#[utoipa::path(
    get,
    path = "/session/{session_id}/state",
    params(
        ("session_id" = String, Path, description = "Unique identifier of the session")
    ),
    responses(
        (status = 200, description = "Session state retrieved successfully", body = crate::types::session_data::SessionData),
        (status = 404, description = "Session not found")
    ),
    tag = "Session Management"
)]
pub async fn get_session_state(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id_path: actix_web::web::Path<std::string::String>,
) -> actix_web::HttpResponse {
    let session_id_str = session_id_path.into_inner();
    let session_id = match uuid::Uuid::parse_str(&session_id_str) {
        std::result::Result::Ok(id) => id,
        std::result::Result::Err(_) => {
            return actix_web::HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Invalid session ID format"}));
        }
    };

    match crate::session::manager::get_session(app_state, &session_id).await {
        std::option::Option::Some(session_data) => {
            actix_web::HttpResponse::Ok().json(session_data)
        }
        std::option::Option::None => actix_web::HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Session not found"})),
    }
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
            time_limit: std::time::Duration::from_secs(300),
            token_threshold: 1000,
            preserve_exchanges: 5,
            initial_instruction: std::option::Option::Some("Test instruction".to_string()),
            compaction_policy: crate::types::compaction_policy::CompactionPolicy::default(),
            evaluation_policy: crate::types::evaluation_policy::EvaluationPolicy::default(),
        }
    }

    #[tokio::test]
    async fn test_get_session_state_success() {
        let app_state = create_test_app_state();
        let config = default_session_config();

        // Create a session directly using the manager for testing setup.
        let session_id = crate::session::manager::create_session(app_state.clone(), config).await;
        let session_id_str = session_id.to_string();

        // Update status to something specific for assertion
        let _ = crate::session::manager::update_status(
            app_state.clone(),
            &session_id,
            crate::types::session_status::SessionStatus::Completed
        ).await;


        let path = actix_web::web::Path::from(session_id_str.clone());
        let resp = super::get_session_state(app_state.clone(), path).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        // Deserialize body and check content
        let body = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
        let session_data: crate::types::session_data::SessionData = serde_json::from_slice(&body).unwrap();

        assert_eq!(session_data.session_id, session_id_str);
         match session_data.status {
            crate::types::session_status::SessionStatus::Completed => {
                // Session completed successfully
            },
            _ => panic!("Unexpected session status"),
        }
        assert!(session_data.history.is_empty()); // Assuming history wasn't modified
    }

    #[tokio::test]
    async fn test_get_session_state_not_found() {
        let app_state = create_test_app_state();
        let non_existent_id = uuid::Uuid::new_v4().to_string();
        let path = actix_web::web::Path::from(non_existent_id);

        let resp = super::get_session_state(app_state, path).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_session_state_invalid_id_format() {
        let app_state = create_test_app_state();
        let invalid_id = "not-a-uuid".to_string();
        let path = actix_web::web::Path::from(invalid_id);

        let resp = super::get_session_state(app_state, path).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }
}