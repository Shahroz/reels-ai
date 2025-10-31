//! Checks if the current session context is sufficient to answer the latest user request.
//!
//! This module defines the `SufficiencyCheckResult` struct and the async function
//! `check_sufficiency_for_answer`. The function interacts with an LLM to
//! evaluate the adequacy of the session's history and gathered context
//! against the most recent user query. Adheres to one-item-per-file guideline.

use llm::llm_typed_unified::output_format::OutputFormat; // Corrected import path
use llm::llm_typed_unified::llm_typed::llm_typed;      // Added import for llm_typed function
use schemars::JsonSchema; // Added import

/// Represents the outcome of the sufficiency check.
///
/// Contains a boolean indicating sufficiency and the LLM's reasoning.
#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, JsonSchema)] // Added JsonSchema derive
pub struct SufficiencyCheckResult {
    /// True if the context is deemed sufficient, false otherwise.
    pub sufficient: bool,
    /// The reasoning provided by the LLM for the sufficiency decision.
    pub reasoning: std::string::String,
}

// Implement FewShotsOutput for the LLM call
impl llm::few_shots_traits::FewShotsOutput<SufficiencyCheckResult> for SufficiencyCheckResult {
    // Corrected trait path, added generic, and renamed function
    fn few_shots() -> Vec<SufficiencyCheckResult> {
        vec![
            SufficiencyCheckResult {
                sufficient: true,
                reasoning: std::string::String::from("The provided context directly addresses the user's question about project status."),
            },
            SufficiencyCheckResult {
                sufficient: false,
                reasoning: std::string::String::from("More information is needed about the specific feature mentioned by the user."),
            },
        ]
    }
}

/// Asynchronously checks if the session data provides sufficient context to answer the latest user request.
///
/// Formats a prompt with relevant session data and queries an LLM to determine sufficiency.
///
/// # Arguments
///
/// * `session_data` - A reference to the current session's data.
/// * `app_state` - Shared application state containing configuration, including LLM models.
///
/// # Returns
///
/// * `Ok(SufficiencyCheckResult)` if the LLM call is successful.
/// * `Err(String)` if there's an error extracting data, formatting the prompt, or during the LLM call.
pub async fn check_sufficiency_for_answer(
    session_data: &crate::types::session_data::SessionData,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
) -> std::result::Result<SufficiencyCheckResult, std::string::String> {
    // Extract the latest user message (assuming ConversationEntry::User variants exist)
    let latest_user_message = session_data
        .history
        .iter()
        .rev()
        // Correctly match on the sender field of the ConversationEntry struct
        .find_map(|entry| match entry.sender {
            crate::types::sender::Sender::User => Some(&entry.message), // Access message field if sender is User
            _ => None,
        })
        .cloned() // Explicitly map and clone the String inside the Option
        .ok_or_else(|| std::string::String::from("No user message found in history."))?;

    // Format conversation history for the prompt
    // Include the last agent message if available to provide context for the user's latest request
    let last_agent_message = session_data
        .history
        .iter()
        .rev()
        .find_map(|entry| match entry.sender {
            crate::types::sender::Sender::Agent | crate::types::sender::Sender::Assistant => Some(&entry.message),
            _ => None,
        })
        .cloned(); // Clone the String inside the Option

    // TODO: Consider adding summarization or truncation for very long histories.
    let history_formatted = session_data
        .history
        .iter()
        .map(|entry| {
            format!(
                "{}: {}",
                match entry.sender {
                    crate::types::sender::Sender::User => "User",
                    crate::types::sender::Sender::Agent => "Agent",
                    crate::types::sender::Sender::Tool => "Tool",
                    crate::types::sender::Sender::System => "System",
                    crate::types::sender::Sender::Assistant => "Assistant",
                },
                entry.message // Assuming message is String
            )
        })
        .collect::<std::vec::Vec<_>>()
        .join("\n");

    // Basic context formatting (can be expanded)
    let context_summary = session_data
        .context
        .iter()
        .map(|ctx| format!("- {}", ctx.content.chars().take(100).collect::<String>())) // Example: take first 100 chars
        .collect::<std::vec::Vec<_>>()
        .join("\n");

    // Format the prompt for the LLM
    // TODO: Refine this prompt significantly for better LLM guidance.
    let prompt = format!(
        "Analyze the conversation flow to determine if enough information exists to fully answer the *latest user request*. Consider the full history, gathered context, and specifically the transition from the *last agent response* to the *latest user request*. Provide your answer as a JSON object with boolean 'sufficient' and string 'reasoning'.

Last Agent Response (if any):
{}

Latest User Request:
{}

Full Conversation History:\n{}\n\nGathered Context:\n{}\n\nIs the available information sufficient? Respond ONLY with the JSON object.",
        last_agent_message.as_deref().unwrap_or("N/A"), // Provide last agent message or N/A
        latest_user_message, // Latest user message
        history_formatted, // Full history
        context_summary // Gathered context
    );

    // Select appropriate models from app_state config
    let models_to_use = &app_state.config.llm_config.context_evaluation_models; // Using context_evaluation_models as specified in LlmConfig
    if models_to_use.is_empty() {
        return std::result::Result::Err(std::string::String::from(
            "No context evaluation models configured.",
        ));
    }

    // Call the LLM using llm_typed_unified
    // Assuming llm_typed takes models, prompt, and maybe other config
    // The exact signature might vary based on the `llm` crate's implementation details.
    let llm_result = llm_typed::<SufficiencyCheckResult>( // Use imported llm_typed function
        prompt, // Pass String prompt
        models_to_use.clone(), // Pass owned Vec<VendorModel>
        3, // Specify retries (e.g., 3)
        Some(OutputFormat::Json), // Specify output format (e.g., JSON)
        false, // Specify debug mode (e.g., false)
    )
    .await; // Assuming llm_typed is async

    // Handle potential errors from the LLM call
    match llm_result {
        std::result::Result::Ok(sufficiency_result) => std::result::Result::Ok(sufficiency_result),
        std::result::Result::Err(e) => std::result::Result::Err(format!("LLM call failed: {}", e)), // Convert error to String
    }
}


#[cfg(test)]
mod tests {
    // Note: Full paths are required for types used in tests as per guidelines.
    // `super::*` is generally discouraged, access items via `super::item_name`.

    #[test]
    fn test_struct_creation() {
        // Basic test to ensure the struct can be created.
        let result = super::SufficiencyCheckResult {
            sufficient: true,
            reasoning: std::string::String::from("Test reasoning"),
        };
        assert!(result.sufficient);
        assert_eq!(result.reasoning, "Test reasoning");
    }

    // TODO: Add mock tests for `check_sufficiency_for_answer`.
    // These tests would require mocking:
    // - `crate::types::session_data::SessionData`
    // - `actix_web::web::Data<crate::state::app_state::AppState>`
    // - The `llm::llm_typed_unified::llm_typed` function call.
    // Example (conceptual):
    // #[actix_rt::test]
    // async fn test_sufficiency_check_logic_mocked() {
    //     // 1. Setup mock SessionData
    //     // 2. Setup mock AppState with mock LlmConfig
    //     // 3. Mock the llm_typed function to return a specific SufficiencyCheckResult
    //     // 4. Call super::check_sufficiency_for_answer(...)
    //     // 5. Assert the result matches the mocked return value.
    //     assert!(true, "Mock tests need implementation");
    // }
}