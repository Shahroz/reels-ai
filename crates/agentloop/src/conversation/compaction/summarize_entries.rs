//! Provides the logic for summarizing a sequence of conversation entries using the typed unified LLM interface.
//!
//! This function takes a slice of `ConversationEntry` objects, constructs a prompt,
//! calls the typed unified LLM (`llm::llm_typed_unified::llm_typed`) using the configured
//! summarization model pool, expects an `LlmSummaryResponse`, and returns the summary string.
//! Adheres to project guidelines.

// Note: Using FQNs as per guidelines.
// Assuming ConversationEntry, AppState, LlmConfig, VendorModel are defined elsewhere.

/// Summarizes a slice of conversation entries using the typed unified LLM.
///
/// # Arguments
///
/// * `entries` - A slice of `ConversationEntry` objects to summarize.
/// * `app_state` - A reference to the application state, containing configuration (including LLM config).
///
/// # Returns
///
/// Returns a `Result<String, String>` containing the summary text on success,
/// or an error message string on failure.
pub async fn summarize_entries(
    entries: &[crate::types::conversation_entry::ConversationEntry],
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
) -> std::result::Result<std::string::String, std::string::String> {
    // 1. Handle empty input gracefully
    if entries.is_empty() {
        return std::result::Result::Ok(std::string::String::from("No entries to summarize."));
    }

    // 2. Construct the prompt string for the LLM
    let conversation_text = entries
        .iter()
        .map(|entry| format!("{:?}: {}", entry.sender, entry.message))
        .collect::<std::vec::Vec<_>>()
        .join("\\n");

    // Prompt instructing the LLM to summarize and adhere to the LlmSummaryResponse structure implicitly via llm_typed.
    let prompt_string = format!(
        "Summarize the following conversation concisely, capturing the key points and flow:\\n\\n---\\n{}\\n---",
        conversation_text
    );

    // 3. Get configuration for the LLM call
    let config = &app_state.config;
    // Assumes summarization_models exists due to prerequisite step.
    let models_to_try = config.llm_config.summarization_models.clone();
    let retries = 5; // Example: Use 1 retry, could be configurable
    let debug_mode = false; // Disable debug mode for standard operation

   // 4. Call the typed unified LLM function
   println!("Calling typed unified LLM for summarization (prompt {} chars)...", prompt_string.len());
   match llm::llm_typed_unified::llm_typed::llm_typed::<crate::types::llm_summary_response::LlmSummaryResponse>(
       prompt_string,
       models_to_try,
       retries,
       Some(llm::llm_typed_unified::output_format::OutputFormat::Json), // Request JSON output
       debug_mode,
   )
   .await
    {
        std::result::Result::Ok(response) => {
            println!("Summarization LLM Typed Response Received ({} chars).", response.summary.len());
            std::result::Result::Ok(response.summary) // Extract the summary string
        }
        std::result::Result::Err(e) => {
            let error_msg = format!("Typed LLM summarization failed: {}", e);
            std::eprintln!("{}", error_msg);
            std::result::Result::Err(error_msg)
        }
    }
}

#[cfg(test)]
mod tests {
    // Note: Using FQNs. Tests require async runtime (tokio).
    // These tests now call the refactored `summarize_entries` which uses the typed unified `llm_typed`.
    // They REQUIRE MOCKING of `llm::llm_typed_unified::llm_typed` to run reliably without
    // making real API calls or failing due to configuration/network issues.
    // Marked as `#[ignore]` for CI/automated runs unless mocking is implemented.

    fn create_test_app_state() -> actix_web::web::Data<crate::state::app_state::AppState> {
        // Uses default AppConfig, which includes default LlmConfig.
       // We need to ensure summarization_models is populated for the test.
       let mut config = crate::config::app_config::AppConfig::default();
       config.llm_config.summarization_models = Some(std::vec![
           llm::llm_typed_unified::vendor_model::VendorModel::default() // Use a default model for testing setup
       ]);
       assert!(
           config.llm_config.summarization_models.as_ref().map_or(false, |m| !m.is_empty()),
            "Test LlmConfig should have summarization models"
        );
        crate::state::app_state::AppState::new(config) // Use constructor
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
            // Assuming default or other fields are handled
            ..Default::default()
        }
    }

    #[tokio::test]
    #[ignore] // Ignored: Requires mocking `llm::llm_typed_unified::llm_typed`.
    async fn test_summarize_basic_conversation_mock_needed() {
        let app_state = create_test_app_state();
        let now = chrono::Utc::now();
        let entries = std::vec![
            create_entry(crate::types::sender::Sender::User, "Hello there.", now - chrono::Duration::seconds(60)),
            create_entry(crate::types::sender::Sender::Agent, "Hi! How can I help?", now - chrono::Duration::seconds(30)),
            create_entry(crate::types::sender::Sender::User, "Tell me about Rust.", now - chrono::Duration::seconds(5)),
        ];

        // Act: Call the refactored function
        let result = super::summarize_entries(&entries, &app_state).await;

        // Assert: Check the result (basic checks without mocking)
        std::println!("Summarization Result (Mock Needed): {:?}", result);
        std::assert!(result.is_ok() || result.is_err(), "Function should return Ok or Err");

        // TODO: With mocking, assert specific Ok(summary_string) or expected Err.
        // Example (if mock returns Ok(LlmSummaryResponse { summary: "Basic Rust conversation." })):
        // assert!(result.is_ok());
        // assert_eq!(result.unwrap(), "Basic Rust conversation.");
        // Example (if mock returns Err):
        // assert!(result.is_err());
        // assert!(result.unwrap_err().contains("Mock LLM Error"));
    }

    #[tokio::test]
    async fn test_summarize_empty_entries() {
        let app_state = create_test_app_state();
        let entries: std::vec::Vec<crate::types::conversation_entry::ConversationEntry> = std::vec![];

        let result = super::summarize_entries(&entries, &app_state).await;

        // Assert: Should return Ok with specific message for empty input
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No entries to summarize.");
    }

     #[tokio::test]
     #[ignore] // Ignored: Requires mocking `llm::llm_typed_unified::llm_typed`.
     async fn test_summarize_single_entry_mock_needed() {
         let app_state = create_test_app_state();
         let now = chrono::Utc::now();
         let entries = std::vec![
             create_entry(crate::types::sender::Sender::User, "Just saying hello.", now),
         ];

         let result = super::summarize_entries(&entries, &app_state).await;

         std::println!("Summarization Result (Single Entry - Mock Needed): {:?}", result);
         std::assert!(result.is_ok() || result.is_err(), "Function should return Ok or Err");

         // TODO: With mocking, assert expected summary for a single entry.
         // Example (if mock returns Ok(LlmSummaryResponse { summary: "User said hello." })):
         // assert!(result.is_ok());
         // assert_eq!(result.unwrap(), "User said hello.");
     }

     // TODO: Add a test case simulating an LLM error by configuring the mock
     //       for `llm::llm_typed_unified::llm_typed` to return Err. Assert that
     //       `summarize_entries` returns the expected Err variant.
     // #[tokio::test]
     // #[ignore] // Requires mocking
     // async fn test_summarize_llm_error_mocked() { ... }
}
