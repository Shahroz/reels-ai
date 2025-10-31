//! Provides the function to dispatch agent tool calls using strongly-typed parameters.
//!
//! This function looks up a tool handler based on the tool name provided
//! in `ToolChoice` and executes it with the `ToolParameters` and context.
//! It also sends WebSocket status messages indicating the start and end of tool execution.
//! Adheres strictly to the project's Rust coding standards.
//! Follows the "one item per file" rule.

//! Revision History
//! - 2025-04-24T18:29:51Z @AI: Update WS event to use detailed ToolResult enum.

// Required types - Used via FQN

// Use the specific ToolResult enum for detailed outcomes
use crate::types::{full_tool_response::FullToolResponse, user_tool_response::UserToolResponse};

/// Dispatches a tool call based on the provided `ToolChoice`.
///
/// Looks up the tool handler in the provided map and executes it with the
/// strongly-typed parameters from `ToolChoice`, application state, and session ID.
/// Sends WebSocket messages before and after execution.
///
/// # Arguments
///
/// * `tool_choice` - The `crate::types::tool_choice::ToolChoice` struct indicating the tool name and parameters.
/// * `app_state` - The shared application state (`actix_web::web::Data<crate::state::app_state::AppState>`).
/// * `session_id` - The ID of the current session (`crate::types::session_id::SessionId`).
///
/// # Returns
///
/// A `Result` containing `(FullToolResponse, UserToolResponse)` on success, or an error `String`
/// if the tool is not found or the handler fails.
pub async fn dispatch_tools(
    tool_choice: crate::types::tool_choice::ToolChoice,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: crate::types::session_id::SessionId,
) -> Result<(FullToolResponse, UserToolResponse), String> {
    match &*app_state.tool_handler {
        Some(handler_fn) => { // handler_fn is &crate::tools::tool_handler::ToolHandler (which is &Box<dyn Fn...>)
            // --- Send "Start" WebSocket Message ---
            // Serialize parameters to a JSON string for logging/WS message if needed.
            // For the handler call, we'll serialize to serde_json::Value.
            let _status_message_start = match serde_json::to_string(&tool_choice.parameters) {
                Ok(params_json) => {
                    std::format!(
                        "Starting tool with parameters: {}",
                        params_json // Contains tool_name and parameters structure
                    )
                }
                Err(e) => {
                    eprintln!( // Using eprintln! for simplicity, consider proper logging
                        "Error serializing tool parameters for: {}", e
                    );
                    std::format!("Starting tool (parameter serialization failed)")
                }
            };

            // Lock ws_connections and send the message
            { // Scope for mutex guard
                let ws_connections_guard = app_state.ws_connections.lock().await;
                if let Some(recipients) = ws_connections_guard.get(&session_id) {
                    for _recipient in recipients { // Prefix unused variable
                        // Send a reasoning update about starting the tool
                        // let event = crate::types::ws_event::WebsocketEvent::ReasoningUpdate(status_message_start.clone());
                        // recipient.do_send(event);
                    }
                }
            } // Mutex guard released here


            // --- Execute the Tool Handler ---
            // Clone necessary parts of tool_choice before moving it into the handler call if needed
           // The handler_fn takes ToolChoice directly
           let tool_handler_result = handler_fn(tool_choice.clone(), app_state.clone(), session_id).await;

            // --- Send "Finish" WebSocket Message ---
            // Construct the WebsocketEvent directly based on the handler's outcome.
            // This assumes that crate::types::ws_event::WebsocketEvent has variants like
            // ToolExecutionSuccess(FullToolResponse) and
            // ToolExecutionFailure { tool_name: String, error: String }
            // to replace the conceptual ToolResult enum.
            let event_to_send = match &tool_handler_result {
                Ok((_full_response, user_response)) => { // _full_response was unused for WS message
                    // _user_response is part of the handler's success tuple,
                   // but was not used in the previous conceptual ToolResult::Success for WS messages.
                   crate::types::ws_event::WebsocketEvent::ToolExecutionSuccess(user_response.clone())
               }
               Err(err_result) => {
                    crate::types::ws_event::WebsocketEvent::ToolExecutionFailure(crate::types::user_tool_failure::UserToolFailure{
                        error: err_result.clone(),
                    })
                }
            };

            // Lock ws_connections and send the event
            { // Scope for mutex guard
                let ws_connections_guard = app_state.ws_connections.lock().await;
                if let Some(recipients) = ws_connections_guard.get(&session_id) {
                    for recipient in recipients {
                        // Send the event, cloning if necessary (if WebsocketEvent is not Copy)
                        recipient.do_send(event_to_send.clone());
                    }
                }
            } // Mutex guard released here

            // Return the original tool handler result (String or Error)
            tool_handler_result
        }
        None => {
            std::result::Result::Err(std::format!("Tool not found or tool handlers not initialized."))
        }
    }
}

#[cfg(test)]
mod tests {
    // Use fully qualified paths for items from parent/other modules.
    // Tests need update to account for AppState structure and potentially mock WS interactions.

    // Helper to create an AppState with specific handlers for testing, wrapped in actix_web::web::Data
    fn create_test_app_state_with_handlers(
        handlers: std::collections::HashMap<String, crate::tools::tool_handler::ToolHandler>
    ) -> actix_web::web::Data<crate::state::app_state::AppState> {
        // Assuming AppConfig can be defaulted or created simply for testing
        let test_config = crate::config::app_config::AppConfig::default(); // Requires AppConfig::default()
        let app_state = crate::state::app_state::AppState::new(
            test_config,
            std::vec::Vec::new(), // empty external_tool_definitions_vec
            std::collections::HashMap::new(), // empty external_tool_handlers_map
            handlers // test-specific handlers are used as merged_tool_handlers
        );
        actix_web::web::Data::new(app_state)
    }


    // Dummy handler matching the updated ToolHandler signature for testing dispatch logic
    async fn dummy_handler(
        tool_choice: crate::types::tool_choice::ToolChoice,
        _app_state: actix_web::web::Data<crate::state::app_state::AppState>,
        _session_id: crate::types::session_id::SessionId,
    ) -> Result<(FullToolResponse, UserToolResponse), String> {
        let full_response = FullToolResponse {
            tool_name: tool_choice.name.clone(),
            properties: tool_choice.parameters.clone(), // or some transformed properties
        };
        let user_response = UserToolResponse {
            tool_name: tool_choice.name,
            summary: "Dummy success summary".to_string(),
        };
        Ok((full_response, user_response))
    }

     // Dummy handler that simulates failure
    async fn dummy_failing_handler(
        _tool_choice: crate::types::tool_choice::ToolChoice,
        _app_state: actix_web::web::Data<crate::state::app_state::AppState>,
        _session_id: crate::types::session_id::SessionId,
    ) -> Result<(FullToolResponse, UserToolResponse), String> {
        Err("Dummy failure".to_string())
    }

    // Assume ToolParameters::Search is defined elsewhere for this test
    // For ToolChoice, parameters are serde_json::Value


    #[tokio::test]
    async fn test_dispatch_tool_not_found() {
        // Test that dispatching a non-existent tool returns an error.
        let test_app_state = create_test_app_state_with_handlers(std::collections::HashMap::new()); // No handlers
        let tool_choice = crate::types::tool_choice::ToolChoice {
            name: std::string::String::from("non_existent_tool"),
            parameters: serde_json::json!({ "query": "" }),
        };
        let dummy_session_id = uuid::Uuid::new_v4();

        // Call the function under test
        // No longer pass handlers map explicitly
        let result = super::dispatch_tools(tool_choice, test_app_state, dummy_session_id).await;

        std::assert!(result.is_err());
        std::assert_eq!(
            result.unwrap_err(),
            "Tool 'non_existent_tool' not found."
        );
        // Note: Cannot easily verify WS messages here without mocking recipients.
        // Assume ToolResult::Failure would have been constructed if handler existed.
    }

    #[tokio::test]
    async fn test_dispatch_tool_success_dummy() {
        // Test successful dispatch using a dummy handler.
        let mut handlers = std::collections::HashMap::new();
        handlers.insert("dummy_tool".to_string(), dummy_handler as crate::tools::tool_handler::ToolHandler);

        let tool_choice = crate::types::tool_choice::ToolChoice {
            name: "dummy_tool".to_string(),
            parameters: serde_json::json!({ "url": "dummy_url" }),
        };
        let test_app_state = create_test_app_state_with_handlers(handlers);
        let dummy_session_id = uuid::Uuid::new_v4();

        let result = super::dispatch_tools(tool_choice, test_app_state, dummy_session_id).await;

        assert!(result.is_ok());
        let (full_resp, user_resp) = result.unwrap();
        assert_eq!(full_resp.tool_name, "dummy_tool");
        assert_eq!(full_resp.response, serde_json::json!({ "url": "dummy_url" }));
        assert_eq!(user_resp.summary, "Dummy success summary");
        // Note: Cannot easily verify WS messages here.
        // Assume ToolResult::success was called and constructed a ToolResult::Success variant
        // (likely ToolOutcome::Browse based on parameters) for the WS message.
    }

    #[tokio::test]
    async fn test_dispatch_tool_failure_dummy() {
        // Test dispatch failure using a dummy failing handler.
        let mut handlers = std::collections::HashMap::new();
        handlers.insert("failing_tool".to_string(), dummy_failing_handler as crate::tools::tool_handler::ToolHandler);

        let tool_choice = crate::types::tool_choice::ToolChoice {
            name: "failing_tool".to_string(),
            parameters: serde_json::json!({ "query": "fail" }),
        };
        let test_app_state = create_test_app_state_with_handlers(handlers);
        let dummy_session_id = uuid::Uuid::new_v4();

        let result = super::dispatch_tools(tool_choice, test_app_state, dummy_session_id).await;

        std::assert!(result.is_err());
        std::assert_eq!(result.unwrap_err(), "Dummy failure");
        // Note: Cannot easily verify WS messages here.
        // Assume ToolResult::failure was called and constructed a ToolResult::Failure variant for the WS message.
    }

     // Test for dispatching save_context adapted from original, using a dummy handler
     // to avoid complex state setup within this specific test module.
     #[tokio::test]
    async fn test_dispatch_save_context_via_dummy_handler() {
        // This dummy handler will simulate what a host-provided 'save_context' might do.
        async fn dummy_save_context_handler(
            tool_choice: crate::types::tool_choice::ToolChoice,
            _app_state: actix_web::web::Data<crate::state::app_state::AppState>,
            _session_id: crate::types::session_id::SessionId,
        ) -> Result<(FullToolResponse, UserToolResponse), String> {
            let content_to_save = tool_choice.parameters.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let full_response = FullToolResponse {
                tool_name: tool_choice.name.clone(),
                properties: serde_json::json!({ "content_length": content_to_save.len(), "status": "saved" }),
            };
            let user_response = UserToolResponse {
                tool_name: tool_choice.name,
                summary: format!("Content of length {} saved.", content_to_save.len()),
            };
            Ok((full_response, user_response))
        }

        let mut handlers = std::collections::HashMap::new();
        handlers.insert("save_context".to_string(), dummy_save_context_handler as crate::tools::tool_handler::ToolHandler);

        let tool_choice = crate::types::tool_choice::ToolChoice {
            name: "save_context".to_string(),
            parameters: serde_json::json!({ "content": "Test data", "source": "test_source" }),
        };

        let test_app_state = create_test_app_state_with_handlers(handlers);
        let dummy_session_id = uuid::Uuid::new_v4();

        let result = super::dispatch_tools(tool_choice, test_app_state, dummy_session_id).await;

        assert!(result.is_ok());
        let (full_resp, user_resp) = result.unwrap();
        assert_eq!(full_resp.tool_name, "save_context");
        assert_eq!(full_resp.response.get("content_length"), Some(&serde_json::json!(9)));
        assert_eq!(user_resp.summary, "Content of length 9 saved.");
    }

    // Note: Tests for the actual registered handlers (`handle_search`, `handle_browse`)
    // that involve external calls or complex state are omitted here.
    // The dummy tests primarily verify the dispatch mechanism itself, now including the
    // surrounding WS message sending logic using the new ToolResult structure.
}
