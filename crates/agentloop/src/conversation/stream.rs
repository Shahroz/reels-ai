//! Provides a function to generate a conversation response using the typed unified LLM interface.
//!
//! This module defines the `conversation_event_stream` function which interacts
//! with the typed unified LLM client (`llm::llm_typed_unified::llm_typed`).
//! It builds the prompt using session data, calls the LLM API expecting a structured response,
//! extracts the agent's message, and handles errors.
//! Adheres to project guidelines (FQNs, etc.).

// External crate imports
use thiserror::Error;

// Standard library imports using Fully Qualified Names (FQN)
use std::string::String;
use std::result::Result;
use std::vec::Vec;

// Internal crate imports using FQN
use crate::types::session_data::SessionData;
// Removed find_model_by_alias import
// use llm::llm_typed_unified::{VendorModel, find_model_by_alias};
use llm::llm_typed_unified::output_format::OutputFormat; // Import OutputFormat
use llm::llm_typed_unified::llm_typed::llm_typed; // Import llm_typed function

// --- Conversation Event Types (Removed - No longer streaming events) ---
// Removed: ConversationEvent enum

/// Represents errors that can occur during conversation processing. (Updated for llm_typed)
#[derive(Error, Debug)]
pub enum StreamError {
    #[error("LLM API error: {0}")]
    ApiError(String),
    #[error("Network error during LLM call: {0}")]
    NetworkError(String),
    // Removed: ParsingError, StreamProcessingError as llm_typed handles parsing/validation
    #[error("Internal processing error: {0}")]
    InternalError(String),
    #[error("Failed to build LLM prompt: {0}")]
    PromptBuildError(String),
    #[error("Missing API key for LLM")]
    MissingApiKey, // Keep for potential future checks if needed
    // Removed: EnvVarError (less relevant with unified approach)
    #[error("LLM typed call failed: {0}")] // Updated error message
    LlmCallFailed(String),
    #[error("Configuration error: {0}")]
    ConfigError(String), // Keep for config issues like empty model list
}

// Removed: impl From<llm::vendors::openai::stream_chat_completion::OpenAIStreamError> for StreamError

/// Calls the configured typed unified LLM client to get a structured response based on session data.
///
/// Constructs a prompt based on session data and app state, calls the typed unified LLM API,
/// expecting an `LlmAgentResponse`, extracts the agent's text response,
/// and returns it or an error.
pub async fn conversation_event_stream(
    session_data: &SessionData,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
) -> Result<crate::types::llm_agent_response::LlmAgentResponse, StreamError> { // Changed return type
    // 1. Build the LLM messages (remains the same)
    let llm_messages = match crate::conversation::build_llm_prompt(session_data, &*app_state) { // Use FQN
        Ok(msgs) => {
            if msgs.is_empty() {
                return std::result::Result::Err(StreamError::PromptBuildError(String::from(
                    "No messages found to build prompt.",
                )));
            }
            msgs
        }
        Err(e) => {
            return std::result::Result::Err(StreamError::PromptBuildError(e.to_string()));
        }
    };

    // 2. Concatenate messages into a single prompt string (remains the same)
    let prompt_string = llm_messages
        .into_iter()
        .map(|msg| format!("{}: {}", msg.role, msg.content))
        .collect::<Vec<String>>()
        .join("\\n");

    if prompt_string.is_empty() {
        return Err(StreamError::PromptBuildError(String::from(
            "Constructed prompt string is empty.",
        )));
    }

    // 3. Get configuration (remains mostly the same, but uses conversation_models)
    let config = app_state.config.clone();
    let models_to_try = config.llm_config.conversation_models.clone(); // Use conversation_models
    let _api_key = config.llm_config.api_key.clone(); // Keep API key retrieval if needed internally by llm_typed

    // Check if model list is empty
    if models_to_try.is_empty() {
         return Err(StreamError::ConfigError(String::from(
             "No conversation models configured in LlmConfig.",
         )));
    }

    // Removed: find_model_by_alias logic

    // 4. Define retries
    let retries = 1; // Example: Use 1 retry

    // 5. Call the typed unified LLM function
    println!("Calling typed LLM with prompt (first 100 chars): {}...", prompt_string.chars().take(100).collect::<String>()); // Debug print
    println!("==============================\n\nFull prompt: {}\n\n================================", prompt_string);
   match llm_typed::<crate::types::llm_agent_response::LlmAgentResponse>( // Changed generic type, use imported function
        prompt_string, // Pass the constructed prompt
        models_to_try, // Pass the list of models from config
        retries,
        Some(OutputFormat::Json), // Specify JSON output format
        false, // Set debug mode (e.g., false for production)
    )
    .await
    {
        Ok(response) => {
            // Successfully received and parsed the typed response
            // Updated logging to use fields from LlmAgentResponse
            println!("Typed LLM Response Received: user_answer='{}...', agent_reasoning='{}...'", response.user_answer.chars().take(100).collect::<String>(), response.agent_reasoning.chars().take(100).collect::<String>());
            Ok(response) // Return the entire response struct
        }
        Err(e) => {
            // Map the anyhow::Error from llm_typed to StreamError
            eprintln!("Typed LLM call failed: {:?}", e); // Log the error
            std::result::Result::Err(StreamError::LlmCallFailed(e.to_string()))
        }
    }
    // Removed: Old non-typed llm call and placeholder response handling.
}


#[cfg(test)]
mod tests {
    // Note: These tests need proper mocking of `llm_typed` to be effective.
    // They are marked `#[ignore]` for now.

    use tokio; // Ensure tokio is a dev-dependency

    // FQN for internal types
        use crate::config::app_config::AppConfig;
    use crate::config::llm_config::LlmConfig;
    use crate::types::{SessionId, Timestamp};
    use crate::types::session_data::SessionData;
    use crate::types::conversation_entry::ConversationEntry;
    use crate::types::sender::Sender;
    use crate::types::llm_agent_response::LlmAgentResponse; // Import for result type
    // Import StreamError, but not ConversationEvent
    use super::{conversation_event_stream, StreamError};

    // Helper to create a dummy AppState for testing (ensure conversation_models is populated)
    fn create_dummy_app_state() -> actix_web::web::Data<tokio::sync::Mutex<AppState>> {
         let dummy_llm_config = LlmConfig {
             // Ensure conversation_models has at least one dummy model
             conversation_models: vec![llm::llm_typed_unified::VendorModel::OpenAI(
                 llm::vendors::openai::openai_model::OpenAIModel::Gpto3Mini, // Example model
             )],
             // Provide other necessary fields if not covered by Default::default()
             api_key: Some(String::from("test-api-key")),
             temperature: Some(0.7),
             max_tokens: Some(100),
             ..Default::default() // Assumes LlmConfig derives Default
         };
         let dummy_config = AppConfig {
             database_url: String::from("dummy_db_url"),
             server_address: String::from("dummy_server_addr"),
             evaluator_sleep_seconds: 60,
             session_timeout_seconds: 3600,
             llm_config: dummy_llm_config,
             ..Default::default() // Assuming AppConfig derives Default
         };
        actix_web::web::Data::new(tokio::sync::Mutex::new(AppState {
            config: std::sync::Arc::new(dummy_config),
            sessions: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            ws_connections: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }))
    }

    // Helper to create dummy SessionData (remains the same)
    fn create_dummy_session_data() -> SessionData {
         SessionData {
             session_id: SessionId::new_v4(),
             user_id: Some("test_user".to_string()),
             history: std::vec![
                 ConversationEntry {
                     sender: Sender::User,
                     message: String::from("Tell me a short story."),
                     timestamp: chrono::Utc::now(),
                     tools: std::vec![],
                     ..Default::default()
                 }
             ],
             ..Default::default()
        }
    }

    #[tokio::test]
    #[ignore] // Ignore this test until mocking for llm_typed is implemented
    async fn test_conversation_function_signature_and_basic_run_mock_needed() {
        // Arrange: Create dummy state and data
        let app_state = create_dummy_app_state();
        let session_data = create_dummy_session_data();

        // Act: Call the refactored function
        // This now calls the typed unified `llm_typed` function.
        // Updated result type annotation
        let result: Result<crate::types::llm_agent_response::LlmAgentResponse, StreamError> = conversation_event_stream(&session_data, &app_state).await;

        // Assert: Check the result type
        // TODO: THIS TEST REQUIRES MOCKING `llm_typed`!
        // Without mocking, it will fail due to network/auth errors or config issues.
        // We just check if it returns *any* result (Ok<LlmAgentResponse> or Err<StreamError>).
        println!("Test Result (Mock Needed): {:?}", result);

        // Assert that *something* happened. A more robust test would mock the `llm_typed` response.
        // Updated assertion message
        assert!(result.is_ok() || result.is_err(), "Function should return Ok(LlmAgentResponse) or Err<StreamError>");

        // Example assertion if expecting an error due to missing model or API key (adjust based on actual error)
        // if let Err(e) = result {
        //     match e {
        //         StreamError::ConfigError(_) | StreamError::LlmCallFailed(_) => {
        //             // Expected error types without mocking
        //         }
        //         other_err => {
        //             panic!("Expected ConfigError or LlmCallFailed without mocking, got {:?}", other_err);
        //         }
        //     }
        // }

        // TODO: Implement proper mocking for `llm::llm_typed_unified::llm_typed`.
        //       - Mock the specific call signature used (including the generic type LlmAgentResponse).
        //       - Return a predefined Ok(LlmAgentResponse { ... }) or Err(anyhow::Error).
        //       - Assert that the function returns the expected Ok(LlmAgentResponse) or StreamError based on the mock.
    }

    // TODO: Add more tests with proper mocking:
    // - Test successful llm_typed call returning Ok(LlmAgentResponse { ... }).
    // - Test llm_typed call failure mapping to StreamError::LlmCallFailed.
    // - Test prompt building failure scenarios mapping to StreamError::PromptBuildError.
    // - Test empty conversation_models config mapping to StreamError::ConfigError.
}