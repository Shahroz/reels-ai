//! Handles the execution of tool calls requested by the LLM.
//!
//! Iterates through the actions proposed in the `LlmAgentResponse`,
//! dispatches each tool call using `dispatch_tools`, processes the result
//! (adding entries with serialized `ToolResult` to history, WS events handled by `dispatch_tools`),
//! and handles errors non-fatally for the loop.
//! Adheres to the one-item-per-file guideline and uses FQNs.

// Required for serde_json serialization
use serde_json;

/// Executes the tool calls specified in the LLM response.
/// Records tool success or failure outcomes (as serialized `ToolResult`) in the conversation history.
///
/// # Arguments
/// * `llm_response` - The response from the LLM, containing potential actions.
/// * `app_state` - Shared application state.
/// * `session_id` - The ID of the current session.
///
/// # Returns
/// * `Ok(())` after processing all tool calls. Individual tool failures are recorded in history
///   but do not cause this function to return `Err`.
/// * `Err(String)` currently not returned, errors are logged and handled internally.

/// Executes the tool calls specified in the LLM response.
pub async fn handle_tool_calls(
    llm_response: &crate::types::llm_agent_response::LlmAgentResponse,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: crate::types::session_id::SessionId,
) -> std::result::Result<(), String> {
    if llm_response.actions.is_empty() {
        log::debug!("No actions requested by LLM for session {}", session_id);
        return std::result::Result::Ok(());
    }

    log::info!(
        "LLM requested {} actions for session {}",
        llm_response.actions.len(),
        session_id
    );


    for tool_choice in &llm_response.actions {
        log::info!(
            "Dispatching tool for session {} with params: {:?}",
            session_id, tool_choice.parameters
        );

        // Dispatch the tool call
        // dispatch_tools handles WS broadcasts internally, including sending structured ToolResult events.
        match crate::tools::dispatch_tools::dispatch_tools(
            tool_choice.clone(), // Pass clone to dispatch
            app_state.clone(),
            session_id, // Pass owned SessionId
        )
        .await
        {
            Ok((full_tool_response, user_tool_response)) => {
                log::info!(
                    "Tool executed successfully for session {}. Output: '{}...'",
                    session_id,
                    user_tool_response.summary // Use summary from UserToolResponse for concise logging
                );

                // Serialize ToolResult to store in history message field
                // Store the full_tool_response (FullToolResponse) in history
                let message_content = serde_json::to_string(&full_tool_response).unwrap_or_else(|e| {
                    log::error!("Failed to serialize FullToolResponse for history: {}", e);
                    // Fallback message if serialization fails
                    format!(
                        "Tool succeeded, but FullToolResponse serialization failed: {}",
                        e
                    )
                });

               // Add tool result entry to history with serialized ToolResult
               let tool_result_entry = crate::types::conversation_entry::ConversationEntry {
                    sender: crate::types::sender::Sender::Tool, // Sender is Tool for success
                    message: message_content, // Store serialized ToolResult JSON
                    timestamp: chrono::Utc::now(),
                    tools: vec![tool_choice.clone()], // Associate the choice
                    id: uuid::Uuid::new_v4(),
                    parent_id: None,
                    depth: 0,
                    attachments: std::vec::Vec::new(), // Tool results typically don't have separate attachments
                    tool_choice: Some(tool_choice.clone()),
                    tool_response: Some(crate::types::tool_response::ToolResponse::Success(full_tool_response)),
                };

                match crate::session::manager::add_conversation_entry(
                    app_state.clone(),
                    &session_id, // Pass reference
                    tool_result_entry,
                )
                .await
                {
                    Ok(_) => log::debug!("Added tool success result (as ToolResult) to history for session {}", session_id),
                    Err(e) => {
                        log::error!(
                            "Failed to add tool success result to history for session {}: {}",
                            session_id, e
                        );
                        // Log and continue even if history add fails
                    }
                }
                 // WebSocket broadcast is handled by dispatch_tools.
            }
            Err(tool_error) => {
                log::error!(
                    "Tool execution failed for session {}: {}",
                    session_id, tool_error
                );

                // Create UserToolFailure for structured error information
                let user_tool_failure = crate::types::user_tool_failure::UserToolFailure {
                    error: tool_error,
                };

                // Serialize UserToolFailure to store in history message field
                let message_content = serde_json::to_string(&user_tool_failure).unwrap_or_else(|e| {
                    log::error!("Failed to serialize UserToolFailure for history: {}", e);
                     // Fallback message if serialization fails
                    format!("Tool failed, and UserToolFailure serialization failed: {}", e)
                });


                // Add tool error entry to history with serialized ToolResult
                let tool_error_entry = crate::types::conversation_entry::ConversationEntry {
                    sender: crate::types::sender::Sender::System, // Sender is System for errors/failures
                    message: message_content, // Store serialized ToolResult JSON
                    timestamp: chrono::Utc::now(),
                    tools: vec![tool_choice.clone()], // Associate the choice
                    id: uuid::Uuid::new_v4(),
                    parent_id: None,
                    depth: 0,
                    attachments: std::vec::Vec::new(), // Tool errors typically don't have separate attachments
                    tool_choice: Some(tool_choice.clone()),
                    tool_response: Some(crate::types::tool_response::ToolResponse::Failure(user_tool_failure)),
                };

                match crate::session::manager::add_conversation_entry(
                    app_state.clone(),
                    &session_id, // Pass reference
                    tool_error_entry,
                )
                .await
                {
                    Ok(_) => log::debug!("Added tool failure result (as ToolResult) to history for session {}", session_id),
                    Err(e) => {
                        log::error!(
                            "Failed to add tool failure result to history for session {}: {}",
                            session_id, e
                        );
                        // Log and continue
                    }
                }
                 // WebSocket broadcast is handled by dispatch_tools.

                // Continue the loop even if a tool fails.
                // If a tool failure should be fatal, return Err here.
            }
        }
    }

    std::result::Result::Ok(())
}


#[cfg(test)]
mod tests {
    // Tests require significant mocking:
    // - dispatch_tools (to return Ok or Err)
    // - add_conversation_entry (to verify the ConversationEntry structure, specifically the serialized message)
    // - Potentially AppState and SessionManager setup.
    // - Need to ensure serde_json is available in test context.

    // Example structure for a test case:
    #[tokio::test]
    async fn test_handle_one_tool_success_stores_serialized_toolresult() {
        // 1. Setup Mocks:
        //    - Mock `dispatch_tools` to return `Ok("Successful tool output")`.
        //    - Mock `add_conversation_entry` to capture the `ConversationEntry` passed to it.
        //    - Create necessary test AppState, SessionId, LlmAgentResponse with one tool choice.
        // 2. Call `handle_tool_calls`.
        // 3. Assert `handle_tool_calls` returned `Ok(())`.
        // 4. Assert `add_conversation_entry` was called once.
        // 5. Get the captured `ConversationEntry`.
        // 6. Assert `entry.sender == Sender::Tool`.
        // 7. Assert `entry.tools` contains the correct tool choice.
        // 8. Deserialize `entry.message` (which should be JSON) back into a `ToolResult`.
        // 9. Assert the deserialized `ToolResult` is `ToolResult::Success` with the expected outcome details
        //    (e.g., `ToolOutcome::Generic` or specific like `Search` based on the tool choice params).
        // 10. Assert the outcome contains the original "Successful tool output".
    }

     #[tokio::test]
    async fn test_handle_one_tool_failure_stores_serialized_toolresult() {
        // 1. Setup Mocks:
        //    - Mock `dispatch_tools` to return `Err("Tool execution failed error")`.
        //    - Mock `add_conversation_entry` to capture the `ConversationEntry`.
        //    - Create test state as above.
        // 2. Call `handle_tool_calls`.
        // 3. Assert `handle_tool_calls` returned `Ok(())`.
        // 4. Assert `add_conversation_entry` was called once.
        // 5. Get the captured `ConversationEntry`.
        // 6. Assert `entry.sender == Sender::System`.
        // 7. Assert `entry.tools` contains the correct tool choice.
        // 8. Deserialize `entry.message` back into a `ToolResult`.
        // 9. Assert the deserialized `ToolResult` is `ToolResult::Failure` with the correct tool name
        //    and the original "Tool execution failed error" message.
    }

    // Add tests for: no tools, multiple tools (mixed success/failure), history add failure (should still return Ok).
}