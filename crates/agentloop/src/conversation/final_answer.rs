//! Generates the final structured answer using an LLM based on collected session context.
//!
//! This function retrieves session data, extracts context, and uses the typed unified LLM
//! (`llm::llm_typed_unified::llm_typed`) to generate a `FinalAnswerResponse`.
//! Adheres to project Rust coding standards including FQNs where feasible.

// Note: Using fully qualified paths as per guidelines, except for traits required by llm_typed/JsonSchema.
// Necessary imports for llm_typed and associated traits.

/// Generates a final structured answer for a given session using the typed unified LLM.
///
/// Fetches the session, processes its context, calls `llm_typed` to generate
/// the response structure, and returns a `FinalAnswerResponse`.
/// Handles cases where the session doesn't exist or the LLM call fails.
///
/// # Arguments
/// * `app_state` - Shared application state containing session data and configuration.
/// * `session_id` - The ID of the session to process.
///
/// # Returns
/// A `Result` containing the `FinalAnswerResponse` or an error message string.
pub async fn generate_final_answer(
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: &crate::types::session_id::SessionId,
) -> std::result::Result<crate::types::final_answer_response::FinalAnswerResponse, std::string::String> {
    log::info!("Generating final answer for session {}", session_id);

    // Fetch session data asynchronously.
    let session_opt = crate::session::manager::get_session(app_state.clone(), session_id).await; // Cloning Data<AppState> is cheap

    // Handle case where session is not found.
    let session_data = match session_opt {
        std::option::Option::Some(data) => data,
        std::option::Option::None => {
            log::warn!("Session not found during final answer generation: {}", session_id);
            return std::result::Result::Err(std::format!("Session not found: {}", session_id));
        }
    };

    // Extract context entries.
    // Assuming SessionData has a `context` field of type Vec<ContextEntry> where ContextEntry has `content: String`.
    let context = session_data.context.clone(); // Assuming context field exists on SessionData

    // Format context into a single string for the LLM prompt.
    let formatted_context_string = if context.is_empty() {
        std::string::String::from("No context available for this session.")
    } else {
        let formatted_context_parts: std::vec::Vec<std::string::String> = context
            .into_iter()
            .map(|entry| entry.content) // Assuming ContextEntry has a content: String field
            .collect();
        // Use double newlines as a common separator for distinct context entries.
        formatted_context_parts.join("\\n\\n") // Ensure correct newline escaping if needed for LLM
    };

    // --- LLM Call using llm_typed ---
    log::debug!("Attempting typed LLM call for final answer generation. Context length: {}", formatted_context_string.len());

    let prompt = std::format!(r#"
        <SESSION_DATA>{:?}</SESSION_DATA>
        <LATEST_USER_QUERY>{}</LATEST_USER_QUERY>

        Please analyze the SESSION_DATA (especially context and history) and based on the LATEST_USER_QUERY, generate the best possible answer in Markdown format.
        Focus on addressing the query directly and comprehensively using only the information available in SESSION_DATA.

        The title of the final answer should be short and to the point one sentence summarizing the research task
        If the research was about a company or a person include it as the first thing.
        "#,
        session_data,
        session_data.history.iter().filter(|e| e.sender == crate::types::sender::Sender::User).last().map_or("No user query found", |m| &m.message) // Extract latest user message
    );

    // Prepare models and configuration for the llm_typed call.
    // TODO: Add a specific model list for final answer generation in `LlmConfig`.
    // Using conversation_models as a placeholder.
    let models_to_try = app_state.config.llm_config.conversation_models.clone();
    let retries = 3; // Example: Use 3 retries, could be configurable
    let debug_mode = false; // Disable debug mode for standard operation

    // Call the typed unified LLM function. It implicitly handles the updated FinalAnswerResponse structure.
    let _llm_result = llm::llm_typed_unified::llm_typed::<crate::types::final_answer_response::FinalAnswerResponse>(
        prompt, // Use the constructed prompt
        models_to_try,
        retries,
        Some(llm::llm_typed_unified::OutputFormat::Json), // Expect JSON output for FinalAnswerResponse
        debug_mode,
    ).await;
    // --- End LLM Call ---
    
    // Process the LLM result.
    match _llm_result { // Use the renamed variable
        std::result::Result::Ok(response) => {
            log::info!("Successfully generated final answer structure for session {}", session_id);
            std::result::Result::Ok(response)
        }
        std::result::Result::Err(err) => {
            log::error!("Typed LLM call failed during final answer generation for session {}: {}", session_id, err);
            // Return the error message string directly as per the function signature.
            std::result::Result::Err(std::format!("Failed to generate final answer via LLM: {}", err))
        }
    }
}

#[cfg(FALSE)] // Keep tests disabled as they require significant mocking of llm_typed.
mod tests {
    // Tests for generate_final_answer would go here.
    // They would need to mock:
    // - `AppState` and its `config`.
    // - `crate::session::manager::get_session` to return mock `SessionData`.
    // - `llm::llm_typed_unified::llm_typed` function itself (using a mocking framework like `mockall`).
    // Test cases should cover:
    // - Success path: llm_typed returns Ok(FinalAnswerResponse { title: "...", markdown_response: "..." }).
    // - LLM error path: llm_typed returns Err.
    // - Session not found path.
    // - Empty context path.
}