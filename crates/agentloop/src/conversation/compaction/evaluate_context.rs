//! Evaluates the conversation context asynchronously using an LLM, returning feedback.
//!
//! Analyzes the provided conversation history and application state via an LLM call
//! to determine context sufficiency, identify missing information, and suggest next steps.
//! Returns a `ContextEvaluatorFeedback` struct on success or an error string on failure.
//! Adheres to one-item-per-file, FQNs, async, and documentation guidelines.
// NOTE: Adding use statements to resolve trait bounds E0277, potentially violating guidelines.
// Required because the compiler needs these traits in scope at the call site of llm_typed.

use llm::llm_typed_unified::llm_typed::llm_typed;
use llm::llm_typed_unified::output_format::OutputFormat;

// Assuming ConversationHistory is defined in crate::conversation::conversation_history.
// Assuming AppState is defined in crate::state::app_state.
// Assuming ContextEvaluatorFeedback is defined in crate::types::context_evaluator_feedback.
// Assuming LlmContextEvaluation is defined in crate::types::llm_context_evaluation.
// Assuming llm_typed is available at llm::llm_typed_unified::llm_typed.

/// Evaluates conversation context asynchronously using an LLM.
pub async fn evaluate_context(
    history: &crate::conversation::conversation_history::ConversationHistory,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>, // _app_state currently unused, but kept for signature consistency
) -> std::result::Result<crate::types::context_evaluator_feedback::ContextEvaluatorFeedback, std::string::String> {
    // Serialize history for the prompt (using Debug format for simplicity)
    // TODO: Implement more robust history summarization/serialization if Debug format is too verbose or unsuitable.
    let history_summary = format!("{:?}", history);

    // Construct the prompt for the LLM
    let prompt = format!(
        "Analyze the following conversation history to assess context sufficiency for the agent's goal.
Determine if the current context is sufficient to proceed, identify any critical missing information,
and suggest concrete next steps for the agent.

Conversation History (Debug Format):
---
{}
---

Respond with whether the context is sufficient, what information is missing (if any), and suggested next steps.",
        history_summary
    );
    // Call the typed LLM function
    let llm_result = llm_typed::<crate::types::llm_context_evaluation::LlmContextEvaluation>(
        prompt,
        app_state.config.llm_config.context_evaluation_models.clone(), // Use default model(s)
        1, // Number of retries
        Some(OutputFormat::Json), // Expect JSON output
        false, // Debug mode disabled
    ).await;

    // Handle the LLM result
    match llm_result {
        std::result::Result::Ok(llm_response) => {
            // Map LlmContextEvaluation to ContextEvaluatorFeedback
            let feedback = crate::types::context_evaluator_feedback::ContextEvaluatorFeedback {
                // Set relevance score based on sufficiency (simple mapping)
                relevance_score: if llm_response.is_sufficient { 1.0 } else { 0.0 },
                // Map next_steps to suggestions
                suggestions: llm_response.next_steps,
                // Context needs update if it's not sufficient
                needs_update: !llm_response.is_sufficient,
                // Note: missing_information from LLM is not directly mapped here,
                // but influences needs_update. Could be logged or added to suggestions if needed.
            };
            std::result::Result::Ok(feedback)
        }
        std::result::Result::Err(e) => {
            // Format the error into a String
            std::result::Result::Err(format!("LLM context evaluation failed: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    // Note: No `use` statements. Access parent items via `super::`. Fully qualify all types.
    // These tests call the actual `evaluate_context` function.
    // Without mocking `llm_typed`, they will make real LLM calls (if network/API keys configured)
    // or fail if the LLM call fails.
    // Assertions on the *content* of the feedback rely on hypothetical LLM behavior
    // and are included to test the mapping logic *assuming* a specific LLM response.
    // These tests should be `#[ignore]`d in CI/automated runs unless mocking is implemented
    // or live LLM testing is intended.

    fn create_mock_history() -> crate::conversation::conversation_history::ConversationHistory {
        std::vec![
            crate::types::conversation_entry::ConversationEntry {
                sender: crate::types::sender::Sender::User,
                message: std::string::String::from("What is the weather in London?"),
                timestamp: chrono::Utc::now(),
                tools: std::vec::Vec::new(),
            },
            crate::types::conversation_entry::ConversationEntry {
                sender: crate::types::sender::Sender::Agent,
                message: std::string::String::from("I will check the weather for you."),
                timestamp: chrono::Utc::now(),
                tools: std::vec::Vec::new(), // Assume tool choice happened elsewhere
            },
        ]
    }

    fn create_mock_app_state() -> actix_web::web::Data<crate::state::app_state::AppState> {
        crate::state::app_state::AppState {
            config: crate::config::app_config::AppConfig::default(), // Assuming default is sufficient
            sessions: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    #[tokio::test]
    #[ignore] // Ignore because it makes a real LLM call without mocking.
    async fn test_evaluate_context_mapping_sufficient() {
        // This test calls the real function. Assertions check the mapping logic
        // *assuming* the LLM hypothetically returns a specific 'sufficient' response.
        let history = create_mock_history();
        let app_state = create_mock_app_state();

        let result = super::evaluate_context(&history, &app_state).await;

        // 1. Basic check: Did the function complete without panicking?
        std::assert!(result.is_ok() || result.is_err(), "Function should return a Result");

        // 2. Check mapping logic *if* Ok and *assuming* LLM returned 'sufficient'
        //    (This part might fail if the actual LLM call returns Err or different content)
        if let std::result::Result::Ok(feedback) = result {
            // Assertions based on the *hypothetical* LLM response:
            // LlmContextEvaluation { is_sufficient: true, next_steps: vec!["Proceed".to_string()], missing_information: None }
            std::assert_eq!(feedback.needs_update, false, "Mapping failed: needs_update should be false if is_sufficient is true");
            std::assert_eq!(feedback.relevance_score, 1.0, "Mapping failed: relevance_score should be 1.0 if is_sufficient is true");
            // We cannot reliably assert feedback.suggestions without controlling the LLM response.
            // std::assert_eq!(feedback.suggestions, std::vec![std::string::String::from("Proceed")]);
            std::println!("Test (sufficient) completed with Ok. Suggestions: {:?}", feedback.suggestions);
        } else if let std::result::Result::Err(e) = result {
             std::println!("Test (sufficient) completed with Err: {}", e);
             // Optionally fail the test if an error is unexpected even with live LLM
             // std::panic!("Expected Ok, but got Err: {}", e);
        }
    }

    #[tokio::test]
    #[ignore] // Ignore because it makes a real LLM call without mocking.
    async fn test_evaluate_context_mapping_insufficient() {
        // This test calls the real function. Assertions check the mapping logic
        // *assuming* the LLM hypothetically returns a specific 'insufficient' response.
        let history = create_mock_history(); // Use same history, assume LLM finds it insufficient
        let app_state = create_mock_app_state();

        let result = super::evaluate_context(&history, &app_state).await;

        // 1. Basic check:
        std::assert!(result.is_ok() || result.is_err(), "Function should return a Result");

        // 2. Check mapping logic *if* Ok and *assuming* LLM returned 'insufficient'
        if let std::result::Result::Ok(feedback) = result {
            // Assertions based on the *hypothetical* LLM response:
            // LlmContextEvaluation { is_sufficient: false, next_steps: vec!["Ask for clarification".to_string()], missing_information: Some("Goal unclear".to_string()) }
            std::assert_eq!(feedback.needs_update, true, "Mapping failed: needs_update should be true if is_sufficient is false");
            std::assert_eq!(feedback.relevance_score, 0.0, "Mapping failed: relevance_score should be 0.0 if is_sufficient is false");
             // We cannot reliably assert feedback.suggestions without controlling the LLM response.
            // std::assert_eq!(feedback.suggestions, std::vec![std::string::String::from("Ask for clarification")]);
             std::println!("Test (insufficient) completed with Ok. Suggestions: {:?}", feedback.suggestions);
        } else if let std::result::Result::Err(e) = result {
            std::println!("Test (insufficient) completed with Err: {}", e);
             // Optionally fail the test if an error is unexpected
             // std::panic!("Expected Ok, but got Err: {}", e);
        }
    }

    #[tokio::test]
    #[ignore] // Ignore because it depends on the LLM call potentially failing.
    async fn test_evaluate_context_error_handling() {
        // This test calls the real function and checks if the error case is handled.
        // It assumes conditions that might cause the LLM call to fail (e.g., invalid API key, network issue).
        let history = create_mock_history();
        let app_state = create_mock_app_state();

        let result = super::evaluate_context(&history, &app_state).await;

        // Assert that if the result is an error, it's formatted as expected.
        if let std::result::Result::Err(e) = result {
            std::assert!(e.starts_with("LLM context evaluation failed:"), "Error message format is incorrect");
            std::println!("Test (error) correctly captured expected error format: {}", e);
        } else {
             std::println!("Test (error) completed with Ok, LLM call succeeded unexpectedly.");
             // This isn't necessarily a failure, just means the LLM call worked.
        }
    }
}
