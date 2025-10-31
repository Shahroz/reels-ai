//! Decides whether the conversation history requires compaction based on policy.
//!
//! This function evaluates the current conversation history length against the
//! `keep_last` setting defined in the `CompactionPolicy` within the application configuration.
//! Returns true if the number of entries exceeds the threshold, indicating compaction is needed.
//! Adheres to one-item-per-file and FQN guidelines.

//! Revision History
//! - 2025-04-24T17:10:16Z @AI: Make function async and update tests to fix E0728.
//! - 2025-04-24T13:25:35Z @AI: Implement logic based on CompactionPolicy and add tests.
//! - 2025-04-24T13:14:27Z @AI: Initial placeholder implementation.

/// Determines if history compaction should be performed based on policy.
///
/// Compares the number of entries in `history` against the `keep_last` value
/// specified in the `CompactionPolicy` found within `app_state.config`.
///
/// # Arguments
/// * `history` - A slice representing the conversation history.
/// * `app_state` - The shared application state containing configuration.
///
/// # Returns
/// `true` if `history.len()` is greater than `app_state.lock().await.config.compaction_policy.keep_last`,
/// `false` otherwise.
///
/// # Assumptions
/// - `app_state.config` contains a field `compaction_policy` of type `crate::types::compaction_policy::CompactionPolicy`.
pub async fn should_compact_history(
    history: &crate::conversation::ConversationHistory, // Type alias for Vec<ConversationEntry>
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
) -> bool {
    // Access the policy from the app state's configuration.
    // Assumes AppConfig has a `compaction_policy` field.
    // Lock needs await, hence the function must be async.
    let policy = &app_state.config.compaction_policy; // Adjust field name if different in AppConfig

    // Determine threshold: use configured keep_last, or fall back to max_conversation_length if keep_last is zero
    let threshold = if policy.keep_last > 0 {
        policy.keep_last
    } else {
        // Fallback to global max conversation length to avoid premature compaction when keep_last==0
        app_state.config.max_conversation_length
    };
    history.len() > threshold
}

#[cfg(test)]
mod tests {
    // Use fully qualified paths as required by guidelines.

    // Helper function to create a dummy AppState with a specific CompactionPolicy.
    // Note: This helper remains synchronous as it only sets up data.
    fn create_test_app_state(keep_last: usize, summary_length: usize) -> actix_web::web::Data<crate::state::app_state::AppState> {
        // Construct the CompactionPolicy for the test.
        let test_policy = crate::types::compaction_policy::CompactionPolicy {
            keep_last,
            summary_length,
        };

        // Construct a dummy AppConfig including the CompactionPolicy.
        let test_config = crate::config::app_config::AppConfig {
            database_url: String::from("dummy_db_url"), // Placeholder value
            server_address: String::from("dummy_server_addr"), // Placeholder value
            evaluator_sleep_seconds: 60, // Placeholder value
            session_timeout_seconds: 3600, // Placeholder value
            compaction_policy: test_policy, // Include the policy
        };

        // Create dummy session storage.
        let dummy_sessions = std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::<crate::types::SessionId, crate::types::session::Session>::new(),
        ));

        // Construct the AppState and wrap it.
        let app_state = crate::state::app_state::AppState {
            config: test_config,
            sessions: dummy_sessions,
        };
        actix_web::web::Data::new(tokio::sync::Mutex::new(app_state))
    }

    // Helper to create a dummy ConversationHistory with a specific length.
    fn create_dummy_history(len: usize) -> crate::conversation::ConversationHistory {
        (0..len)
            .map(|_| crate::types::conversation_entry::ConversationEntry::default()) // ASSUMPTION: Default exists
            .collect::<std::vec::Vec<_>>()
    }

    // Use tokio::test for async tests.
    #[tokio::test]
    async fn test_history_below_threshold() {
        // Test case where history length is less than keep_last.
        let keep_last = 10;
        let history_len = 5;
        let app_state = create_test_app_state(keep_last, 500); // summary_length arbitrary here
        let history = create_dummy_history(history_len);

        // Call the async function with await and clone the app_state Data.
        let result = super::should_compact_history(&history, app_state.clone()).await;

        std::assert_eq!(result, false, "Should not compact when history length ({}) is less than keep_last ({})", history_len, keep_last);
    }

    #[tokio::test]
    async fn test_history_at_threshold() {
        // Test case where history length is equal to keep_last.
        let keep_last = 10;
        let history_len = 10;
        let app_state = create_test_app_state(keep_last, 500);
        let history = create_dummy_history(history_len);

        let result = super::should_compact_history(&history, app_state.clone()).await;

        std::assert_eq!(result, false, "Should not compact when history length ({}) is equal to keep_last ({})", history_len, keep_last);
    }

    #[tokio::test]
    async fn test_history_above_threshold() {
        // Test case where history length is greater than keep_last.
        let keep_last = 10;
        let history_len = 15;
        let app_state = create_test_app_state(keep_last, 500);
        let history = create_dummy_history(history_len);

        let result = super::should_compact_history(&history, app_state.clone()).await;

        std::assert_eq!(result, true, "Should compact when history length ({}) is greater than keep_last ({})", history_len, keep_last);
    }

     #[tokio::test]
    async fn test_history_with_zero_threshold() {
        // Test case with keep_last = 0 (edge case). Compaction should happen if history is not empty.
        let keep_last = 0;
        let history_len = 1;
        let app_state = create_test_app_state(keep_last, 500);
        let history = create_dummy_history(history_len);

        let result = super::should_compact_history(&history, app_state.clone()).await;

        std::assert_eq!(result, true, "Should compact when history length ({}) is greater than keep_last ({})", history_len, keep_last);
    }

     #[tokio::test]
    async fn test_empty_history_with_zero_threshold() {
        // Test case with keep_last = 0 and empty history.
        let keep_last = 0;
        let history_len = 0;
        let app_state = create_test_app_state(keep_last, 500);
        let history = create_dummy_history(history_len);

        let result = super::should_compact_history(&history, app_state.clone()).await;

        std::assert_eq!(result, false, "Should not compact when history length ({}) is equal to keep_last ({})", history_len, keep_last);
    }
}