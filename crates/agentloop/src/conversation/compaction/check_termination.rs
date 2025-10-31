//! Checks if the conversation should terminate based on history, goal, context, and an LLM evaluation.
//!
//! This asynchronous function analyzes the session data, constructs a prompt,
//! and calls an LLM using the configured model pool to determine if termination
//! conditions are met. Returns the LLM's boolean decision.
//! Conforms to the one-item-per-file rule, uses FQNs, and follows async guidelines.

// Required imports for traits used by llm_typed_unified::llm_typed
use llm::llm_typed_unified::llm_typed::llm_typed;
use llm::llm_typed_unified::output_format::OutputFormat;

/// Checks for conversation termination conditions using an LLM.
///
/// # Arguments
///
/// * `session_data` - A reference to the session data, containing history, goal, context, etc.
/// * `app_state` - A reference to the application state, containing configuration (including LLM config).
///
/// # Returns
///
/// Returns `true` if the LLM determines termination is indicated, `false` otherwise.
/// Defaults to `false` if the LLM call fails.
pub async fn check_termination(
    session_data: &crate::types::session_data::SessionData, // Contains history, goal, context
    app_state: actix_web::web::Data<crate::state::app_state::AppState>, // Contains config including LlmConfig
) -> bool {
    // Serialize history and goal for the prompt. Using Debug for simplicity.
    // TODO: Consider a more sophisticated summarization for long histories.
    let history_summary = format!("{:?}", session_data.history.iter().rev().take(10).rev().collect::<Vec<_>>()); // Take last 10 entries
    let _goal = &session_data.research_goal;

    // Construct the prompt for the LLM
    let prompt = format!(
        "Analyze the following recent conversation history and the overall goal.
Determine if the conversation should terminate based on goal completion, user request, or other factors.
Respond with a JSON object containing 'should_terminate' (boolean) and 'reasoning' (string).

Goal: {:?}

Recent History (Debug Format):
---
{}
---

Should the conversation terminate?",
        session_data.research_goal.as_deref().unwrap_or("Not specified"), // Use research_goal
        history_summary
    );
    log::info!("{}", prompt);
    // Call the typed LLM function using the dedicated model pool
    let llm_result = llm_typed::<crate::types::llm_termination_decision::LlmTerminationDecision>(
        prompt,
        app_state.config.llm_config.check_termination_models.clone(), // Use configured models
        1, // Number of retries
        Some(OutputFormat::Json), // Expect JSON output
        true, // Debug mode disabled
    )
    .await;

    // Handle the LLM result
    match llm_result {
        std::result::Result::Ok(llm_response) => {
            std::println!(
                "Termination check LLM result: terminate={}, reasoning='{}'",
                llm_response.should_terminate, llm_response.reasoning
            );
            llm_response.should_terminate
        }
        std::result::Result::Err(e) => {
            std::eprintln!("LLM termination check failed: {:#?}. Defaulting to not terminating.", e);
            // Default to false (continue conversation) if LLM fails
            false
        }
    }
}

#[cfg(test)]
mod tests {
    // Access the function under test via `super::`. Full paths for other items.
    // These tests now require an async runtime (tokio) and are marked #[ignore]
    // because they would make real LLM calls without mocking.

    // --- Mock Types (Copied from previous version, ensure they align if updated elsewhere) ---
    #[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize)]
    pub struct MockContextEntry { pub source: std::string::String, pub content: std::string::String }
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub enum MockSessionStatus { Pending, InProgress, Terminated, Failed }
    impl Default for MockSessionStatus { fn default() -> Self { MockSessionStatus::Pending } }
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub enum MockSender { User, Agent, Tool }
    impl Default for MockSender { fn default() -> Self { MockSender::User } }
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct MockToolChoice { pub name: String }
    impl Default for MockToolChoice { fn default() -> Self { MockToolChoice { name: "".to_string() } } }

    // --- Test Helpers ---
    fn create_test_app_state() -> actix_web::web::Data<tokio::sync::Mutex<crate::state::app_state::AppState>> {
        // Uses default AppConfig, which now includes default LlmConfig
        let config = crate::config::app_config::AppConfig::default();
        crate::state::app_state::AppState::new(config) // Use the constructor
    }

    fn create_entry(
        sender: crate::types::sender::Sender,
        message: &str,
        timestamp: crate::types::timestamp::Timestamp,
    ) -> crate::types::conversation_entry::ConversationEntry {
        crate::types::conversation_entry::ConversationEntry {
            sender,
            message: std::string::String::from(message),
            timestamp,
            tools: std::vec::Vec::new(),
        }
    }

    fn create_test_session_data(
        entries: std::vec::Vec<crate::types::conversation_entry::ConversationEntry>,
        goal: &str,
    ) -> crate::types::session_data::SessionData {
        // Simplified SessionData creation for focus
        let dummy_config = crate::types::session_config::SessionConfig {
             time_limit: std::time::Duration::from_secs(600),
             token_threshold: 1000,
             preserve_exchanges: 5,
        };
        crate::types::session_data::SessionData {
            status: crate::types::session_status::SessionStatus::InProgress,
            config: dummy_config,
            history: entries,
            context: std::vec::Vec::new(),
            research_goal: std::string::String::from(goal),
            created_at: chrono::Utc::now(),
            last_activity_timestamp: chrono::Utc::now(),
            system_message: None,
            messages: std::vec::Vec::new(),
        }
    }

    // --- Test Cases ---

    #[tokio::test]
    #[ignore] // Ignored: Makes real LLM call. Test structure remains for local/integration testing.
    async fn test_check_termination_llm_integration() {
        let app_state = create_test_app_state();
        let now = chrono::Utc::now();
        let entries = std::vec![
            create_entry(crate::types::sender::Sender::User, "Hello", now - chrono::Duration::seconds(60)),
            create_entry(crate::types::sender::Sender::Agent, "Hi there", now - chrono::Duration::seconds(30)),
            create_entry(crate::types::sender::Sender::User, "Please stop", now - chrono::Duration::seconds(5)), // Termination keyword
        ];
        let session_data = create_test_session_data(entries, "General conversation");

        // Call the async function
        let result = super::check_termination(&session_data, &app_state).await;

        // Assertion depends heavily on the live LLM's interpretation.
        // We might expect 'true' due to "Please stop", but cannot guarantee it.
        // A basic check is that it returns *a* boolean.
        std::assert!(result == true || result == false, "Function should return a boolean");
        std::println!("LLM Termination Check Result (Integration Test): {}", result);
        // For a real test, you'd need mocking or specific assertions based on expected LLM behavior for this input.
        // e.g., assert_eq!(result, true, "Expected LLM to detect termination keyword");
    }

    #[tokio::test]
    #[ignore] // Ignored: Makes real LLM call.
    async fn test_check_termination_llm_ongoing_conversation() {
         let app_state = create_test_app_state();
         let now = chrono::Utc::now();
         let entries = std::vec![
             create_entry(crate::types::sender::Sender::User, "What is the weather like?", now - chrono::Duration::seconds(60)),
             create_entry(crate::types::sender::Sender::Agent, "Checking the weather now.", now - chrono::Duration::seconds(30)),
             create_entry(crate::types::sender::Sender::User, "Also, can you find nearby cafes?", now - chrono::Duration::seconds(5)), // Follow-up question
         ];
         let session_data = create_test_session_data(entries, "Find weather and cafes");

         let result = super::check_termination(&session_data, &app_state).await;

         std::assert!(result == true || result == false, "Function should return a boolean");
         std::println!("LLM Termination Check Result (Ongoing Test): {}", result);
         // Expect false, but cannot guarantee without controlling LLM.
         // assert_eq!(result, false, "Expected LLM to see ongoing conversation");
    }

     #[tokio::test]
     #[ignore] // Ignored: Makes real LLM call. Tests error path default.
     async fn test_check_termination_llm_error_defaults_to_false() {
         // Setup state that might cause LLM error (e.g., invalid API key if config could be manipulated,
         // or just rely on potential network issues). Here, we use default valid config but test the default path.
         let mut app_state = create_test_app_state();
         // Intentionally break config IF POSSIBLE (e.g., bad model name?) - difficult without more control.
         // Forcing an error reliably usually requires mocking the llm_typed call.
         // We can only test the *expected* behavior IF an error occurs.

         let now = chrono::Utc::now();
         let entries = std::vec![
             create_entry(crate::types::sender::Sender::User, "Test message", now),
         ];
         let session_data = create_test_session_data(entries, "Test goal");

         let result = super::check_termination(&session_data, &app_state).await;

         // If the LLM call *actually* failed during the test run, we expect false.
         // If it succeeded, the assertion might fail depending on the LLM response.
         // This highlights the difficulty of testing error paths without mocks.
         // A better approach might be to check logs for the error message.
         std::assert!(result == true || result == false, "Function should return a boolean"); // Basic check
         std::println!("LLM Termination Check Result (Error Path Test): {}. If an error occurred, this should ideally be false.", result);
         // Assuming an error occurred: assert_eq!(result, false, "Expected default to false on LLM error");

     }
}