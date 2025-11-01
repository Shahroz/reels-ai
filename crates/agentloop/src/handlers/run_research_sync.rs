//! Handles the request to run a research session synchronously.
//!
//! Accepts a JSON payload, creates a session, runs the research loop
//! in the foreground until a final answer is produced, and returns the
//! full conversation history.
//! Adheres to the one-item-per-file and FQN guidelines.

/// Handles POST requests to run a research session synchronously.
///
/// This endpoint is intended for internal or automated systems that require
/// a direct response rather than polling for status.
///
/// # Arguments
///
/// * `request_payload` - JSON containing the user's research instruction.
/// * `app_state` - Shared application state containing session storage.
///
/// # Returns
///
/// * `HttpResponse` - OK (200) with the JSON-serialized conversation history on success,
///   or InternalServerError (500) if the research process fails.
#[utoipa::path(
    post,
    path = "/research/run-sync",
    request_body = crate::types::research_request::ResearchRequest,
    responses(
        (status = 200, description = "Research completed successfully", body = Vec<crate::types::conversation_entry::ConversationEntry>),
        (status = 500, description = "Internal server error during research process")
    ),
    tag = "Internal"
)]
pub async fn run_research_sync(
    request_payload: actix_web::web::Json<crate::types::research_request::ResearchRequest>,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
) -> impl actix_web::Responder {
    log::debug!("Received run_research_sync request: {:?}", request_payload);

    let session_config = crate::types::session_config::SessionConfig {
        time_limit: std::time::Duration::from_secs(app_state.config.session_timeout_seconds),
        token_threshold: 100000,
        preserve_exchanges: 10,
        initial_instruction: Some(request_payload.instruction.clone()),
        compaction_policy: crate::types::compaction_policy::CompactionPolicy::default(),
        evaluation_policy: crate::types::evaluation_policy::EvaluationPolicy::default(),
    };

    // Create the session
    let session_id = crate::session::manager::create_session(
        app_state.clone(),
        session_config,
    )
    .await;

    // Format and add initial message
    let initial_message_content = crate::utils::message_formatter::format_message_with_attachments(
        &request_payload.instruction,
        request_payload.attachments.as_ref(),
    );
    let initial_entry = crate::types::conversation_entry::ConversationEntry {
        sender: crate::types::sender::Sender::User,
        message: initial_message_content.clone(),
        timestamp: chrono::Utc::now(),
        tools: Vec::new(),
        id: uuid::Uuid::new_v4(),
        parent_id: None,
        depth: 0,
        attachments: request_payload.attachments.clone().unwrap_or_else(std::vec::Vec::new),
        tool_choice: None,
        tool_response: None,
    };

    if let Err(e) =
        crate::session::manager::add_conversation_entry(app_state.clone(), &session_id, initial_entry)
            .await
    {
        log::error!(
            "Failed to add initial conversation entry for sync session {}: {}",
            session_id,
            e
        );
        return actix_web::HttpResponse::InternalServerError().finish();
    }

    // Run the research loop synchronously and wait for the result
    match crate::evaluator::run_research_loop_sync::run_research_loop_sync(
        app_state.clone(),
        session_id,
        initial_message_content,
        None, // No progress callback for sync handler
    )
    .await
    {
        Ok(history) => {
            log::info!("Sync research task for session {} completed successfully.", session_id);
            actix_web::HttpResponse::Ok().json(history)
        }
        Err(e) => {
            log::error!("Sync research task for session {} failed: {}", session_id, e);
            actix_web::HttpResponse::InternalServerError().body(e)
        }
    }
}

#[cfg(test)]
mod tests {
    // Basic placeholder for tests. A full implementation would require
    // mocking the LLM and tool interactions within `run_research_loop_sync`.
}