//! Handles a single turn of interaction with the LLM.
//!
//! Calls the `conversation_event_stream` function, processes the response by
//! adding the agent's message to history, and broadcasts intermediate WebSocket events
//! (reasoning, intermediate answer). Returns the LLM response for further processing (tool calls).
//! Adheres to the one-item-per-file guideline and uses FQNs.

/// Processes one turn of LLM interaction.
///
/// # Arguments
/// * `session_data` - The current data for the session.
/// * `app_state` - Shared application state.
/// * `session_id` - The ID of the current session.
///
/// # Returns
/// * `Ok(LlmAgentResponse)` if the LLM call was successful.
/// * `Err(String)` if the LLM call failed.
pub async fn process_llm_turn(
    session_data: &crate::types::session_data::SessionData,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: &crate::types::session_id::SessionId,
) -> std::result::Result<crate::types::llm_agent_response::LlmAgentResponse, String> {
    log::info!("Calling LLM for session {}", session_id);

    // Validate that a goal is present before proceeding.
    // Check only for research_goal, as initial_instruction is not in SessionConfig.
    if session_data.research_goal.is_none() {
        let error_msg = format!(
            "Cannot process LLM turn for session {}: No research goal found.",
            session_id
        );
        log::error!("{}", error_msg);
        return std::result::Result::Err(error_msg);
    }

    match crate::conversation::stream::conversation_event_stream(
        session_data, // Pass reference
        app_state.clone(),
    )
    .await    {
        Ok(mut llm_response) => { // Make response mutable
            // <<< START INSERTED LOGGING >>>
            log::debug!(
                "process_llm_turn: Received LlmAgentResponse with actions: {:?}",
                llm_response.actions
            );
            // <<< END INSERTED LOGGING >>>

            // Enforce that final answers do not have actions.
            if llm_response.is_final {
                 // Clear actions unconditionally if the response is marked as final.
                llm_response.actions.clear();
            }
            // Proceed with processing the (potentially modified) response...

            log::info!(
                "LLM response for session {}: User Answer: '{}...', Agent Reasoning: '{}...', Full answer: {:#?}",
                session_id,
                llm_response.user_answer.chars().take(100).collect::<String>(),
                llm_response.agent_reasoning.chars().take(100).collect::<String>(),
                &llm_response // Log reference to avoid move
            );

           // Add Agent response to conversation history
           let agent_entry = crate::types::conversation_entry::ConversationEntry {
                sender: crate::types::sender::Sender::Agent,
                message: llm_response.user_answer.clone(),
                timestamp: chrono::Utc::now(),
                tools: llm_response.actions.clone(),
                id: uuid::Uuid::new_v4(),
                parent_id: None,
                depth: 0,
                attachments: std::vec::Vec::new(), // Agent responses typically don't have new attachments by default
                tool_choice: None, // Agent's textual response is not a direct tool choice itself
                tool_response: None, // This is an agent response, not a tool's response
            };
            // Use session_id reference
            match crate::session::manager::add_conversation_entry(
                app_state.clone(),
                session_id,
                agent_entry,
            )
            .await
            {
                Ok(_) => log::debug!("Added agent response to history for session {}", session_id),
                Err(e) => {
                    // Log and continue, history add failure might not be fatal here
                    log::error!(
                        "Failed to add agent response to history for session {}: {}",
                        session_id, e
                    );
                }
            }

            // Broadcast intermediate events via WebSocket
            // Use session_id reference
            broadcast_agent_answer(&app_state, session_id, &llm_response).await;

            std::result::Result::Ok(llm_response)
        }
        Err(e) => {
            log::error!("LLM call failed for session {}: {}", session_id, e);
            // Propagate the error string
            std::result::Result::Err(format!("LLM call failed: {}", e))
        }
    }
}

/// Helper function to broadcast ReasoningUpdate and, if final, the ResearchAnswer WebSocket event.
async fn broadcast_agent_answer(
    app_state: &actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: &crate::types::session_id::SessionId,
    llm_response: &crate::types::llm_agent_response::LlmAgentResponse,
) {
    // Acquire the lock once
    let ws_lock = app_state.ws_connections.lock().await;
    let recipients_clone = if let Some(recipients) = ws_lock.get(session_id) {
        // Clone the list of recipients
        let clone = recipients.clone();
        // Drop the lock *before* sending messages to avoid holding it during potentially slow operations
        drop(ws_lock);
        if clone.is_empty() {
            log::warn!("No recipients found for WebSocket broadcast for session {}", session_id);
            None // Return None if the cloned list is empty
        } else {
            Some(clone) // Return the cloned list
        }
    } else {
        // Lock is automatically dropped here as ws_lock goes out of scope
        log::warn!("No recipient list found for WebSocket broadcast for session {}", session_id);
        None // No recipients found
    };

    // Proceed with broadcasting only if we have recipients
    if let Some(recipients) = recipients_clone {
        use crate::types::ws_event::WebsocketEvent;

        // 1. Reasoning update
        let reasoning_event = WebsocketEvent::ReasoningUpdate(llm_response.agent_reasoning.clone());
        if !recipients.is_empty() {
               log::debug!("Broadcasting ReasoningUpdate to {} recipients for session {}", recipients.len(), session_id);
            for rec in &recipients { // Iterate over the cloned list
                rec.do_send(reasoning_event.clone());
            }
        } else {
             // This case should ideally not be reached due to the check above, but kept for safety.
             log::warn!("Cloned recipient list is empty for ReasoningUpdate broadcast for session {}", session_id);
        }

        // 2. Final Research Answer (only if the response is marked as final)
        if llm_response.is_final {
            let final_answer_event = WebsocketEvent::ResearchAnswer {
                title: llm_response.title.clone().unwrap_or_else(|| "Research Result".to_string()),
                content: llm_response.user_answer.clone(),
            };
            if !recipients.is_empty() {
                log::debug!("Broadcasting ResearchAnswer event to {} recipients for session {}", recipients.len(), session_id);
                for rec in &recipients { // Iterate over the same cloned list
                    rec.do_send(final_answer_event.clone());
                }
            } else {
                 // This case should ideally not be reached.
                 log::warn!("Cloned recipient list is empty for ResearchAnswer broadcast for session {}", session_id);
            }
         }
    }
    // No need for explicit drops here, the lock was handled earlier.
}

#[cfg(test)]
mod tests {
    // Tests require mocking conversation_event_stream, add_conversation_entry, and WebSocket interactions.
    #[tokio::test]
    async fn test_llm_turn_success_no_tools() {
        // Setup mocks: stream returns Ok(response_no_tools), history add Ok, WS mocks
        // Call process_llm_turn
        // Assert Ok(response_no_tools) is returned
        // Verify history add and WS broadcasts (ReasoningUpdate, AgentAnswer) were called
    }

    #[tokio::test]
    async fn test_llm_turn_success_with_tools() {
       // Setup mocks: stream returns Ok(response_with_tools), history add Ok, WS mocks
       // Call process_llm_turn
       // Assert Ok(response_with_tools) is returned
       // Verify history add and WS broadcasts were called
    }

    #[tokio::test]
    async fn test_llm_turn_failure() {
        // Setup mocks: stream returns Err(...)
        // Call process_llm_turn
        // Assert Err(...)
        // Verify history add and WS broadcasts were NOT called
    }

     #[tokio::test]
    async fn test_llm_turn_history_add_failure() {
        // Setup mocks: stream returns Ok(response), history add Err(...), WS mocks
        // Call process_llm_turn
        // Assert Ok(response) is returned // Should still succeed even if history add fails
        // Verify WS broadcasts were called
    }
}
