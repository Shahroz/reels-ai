//! Dispatches agent tool requests to appropriate handlers from the agentloop.
//!
//! This function acts as a ToolHandler for the reels_app backend,
//! routing ToolChoice requests to specific tool implementations within the
//! agentloop crate. It validates parameters and calls the respective agentloop tool.
//! Adheres strictly to project Rust coding standards: one item per file, fully qualified paths.

// Note: No 'use' statements allowed. All paths must be fully qualified.
// Assumes 'agentloop' is a crate dependency and its types are accessible via 'agentloop::...'.
// Assumes 'actix_web' and 'serde_json' are available dependencies.
// Removed: use crate::GLOBAL_POOL; // Will use crate::main::get_global_pool().await instead

pub fn dispatch_reels_agent_tool(
    tool_choice: agentloop::types::tool_choice::ToolChoice,
    app_state: actix_web::web::Data<agentloop::state::app_state::AppState>,
    session_id: agentloop::types::session_id::SessionId,
) -> std::pin::Pin<
    std::boxed::Box<
        dyn std::future::Future<
                Output = std::result::Result<
                    (
                        agentloop::types::full_tool_response::FullToolResponse,
                        agentloop::types::user_tool_response::UserToolResponse,
                    ),
                    std::string::String,
                >,
            > + Send,
    >,
> {
    std::boxed::Box::pin(async move {
        // Parse tool parameters directly from the tool choice
        let reels_tool_params_result: std::result::Result<
            crate::agent_tools::reels_tool_parameters::ReelsToolParameters,
            serde_json::Error,
        > = serde_json::from_value(tool_choice.parameters.clone());

        match reels_tool_params_result {
            std::result::Result::Ok(parsed_tool_params) => {
                // Successfully parsed into a specific tool's parameter structure.
                // Now, dispatch to the correct handler based on the enum variant.
                // Only reel generation related tools are handled.
                match parsed_tool_params {
                  crate::agent_tools::reels_tool_parameters::ReelsToolParameters::BrowseWithQuery(p) => {
                    // Generate a placeholder UUID for user_id parameter (kept for API compatibility)
                    let placeholder_user_id = uuid::Uuid::new_v4();
                    crate::agent_tools::handlers::handle_reels_browse_with_query::handle_reels_browse_with_query(p, placeholder_user_id).await
                  }
                crate::agent_tools::reels_tool_parameters::ReelsToolParameters::GenerateReel(params) => {
                    let gcs_client = crate::services::gcs::gcs_client::GCSClient::new();
                    // Generate a placeholder UUID for user_id (used for GCS path generation)
                    let placeholder_user_id = uuid::Uuid::new_v4();
                    crate::agent_tools::handlers::handle_generate_reel::handle_generate_reel(params, &gcs_client, placeholder_user_id).await
                }
               }
            }
            std::result::Result::Err(e) => {
                // Failed to deserialize parameters for a known tool.
                std::result::Result::Err(format!(
                    "Failed to parse parameters for tool : {}. Parameters (json): {}",
                    e,
                    serde_json::to_string(&tool_choice.parameters)
                        .unwrap_or_else(|_| std::string::String::from("Unserializable parameters"))
                ))
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    // For items from the parent module (this file), use `super::`.
    // For all other items (std::, agentloop::, crate:: for other modules), use FQN.
    // `actix_rt::test` is used for async tests. Ensure `actix-rt` is a dev-dependency.
    // `uuid` crate for generating test UUIDs. Ensure it's a dev-dependency.

    // Mocking/constructing agentloop types (AppState, SessionId, ToolChoice) is necessary for tests.
    // These helpers are used across multiple tests.
    // This assumes these types have public constructors or factory methods usable for testing.
    // For example, `AppState::new_mock()` or similar might be needed.
    // If `agentloop::types::tool_choice::ToolChoice` fields are not public, constructing it for tests will require a helper or public constructor.

    fn create_mock_app_state() -> actix_web::web::Data<agentloop::state::app_state::AppState> {
        // This is a placeholder. A real AppState might be complex.
        // Assuming AppState has a way to be instantiated for tests, e.g. a `new_mock()` method.
        // If AppState is just a struct with public fields, it can be constructed directly.
        // Replace with actual AppState construction logic for tests.
        // Using AppState::new with default/empty values.
        actix_web::web::Data::new(agentloop::state::app_state::AppState::new(
            agentloop::config::app_config::AppConfig::default(), // config
            None,                                                // tool_schemas
            None,                                                // tool_handler
        ))
    }

    fn create_mock_session_id() -> agentloop::types::session_id::SessionId {
        uuid::Uuid::nil() // Use Uuid::nil() for a mock session ID, as SessionId is likely a type alias for uuid::Uuid
    }

    fn create_mock_tool_choice(
        _name: &str,
        params: serde_json::Value,
    ) -> agentloop::types::tool_choice::ToolChoice {
        agentloop::types::tool_choice::ToolChoice { parameters: params }
    }
}

