//! Service for orchestrating agent research tasks.
//!
//! This service handles the setup of the agentloop state and the execution
//! of research tasks, including logging their output to Google Cloud Storage.

use agentloop::evaluator::run_research_loop_sync::run_research_loop_sync;
use agentloop::session::manager as session_manager;
use agentloop::tools::tools_schema::ToolsSchema;

/// Creates and configures the `AppState` for the agentloop.
///
/// This function encapsulates the logic for setting up the tools and configuration
/// required by the agentloop, moving it out of the main server setup.
pub async fn create_agentloop_state() -> std::result::Result<actix_web::web::Data<agentloop::state::app_state::AppState>, std::string::String> {
    let tools_schema = schemars::schema_for!(crate::agent_tools::reels_tool_parameters::ReelsToolParameters);
    let tools_schema_value = ToolsSchema {
        schema: serde_json::to_value(tools_schema).map_err(|e| format!("Cannot convert schema to JSON value: {e}"))?,
    };
    let tool_handler = crate::agent_tools::dispatch_reels_agent_tool::dispatch_reels_agent_tool;

    match agentloop::app_setup::configure_app(Some(tools_schema_value), Some(tool_handler)).await {
        Ok(state) => {
            log::info!("AgentLoop state initialized successfully.");
            Ok(actix_web::web::Data::new(state))
        }
        Err(e) => {
            log::error!("FATAL: Failed to initialize AgentLoop state: {e}");
            Err(format!("Failed to initialize AgentLoop state: {e}"))
        }
    }
}

/// Runs a research task and logs the entire conversation history to GCS.
///
/// This function orchestrates the following steps:
/// 1. Creates a fresh `AppState` for the agentloop.
/// 2. Executes the research task using the provided request.
/// 3. Serializes the resulting conversation history.
/// 4. Uploads the serialized history as a JSON file to a private GCS bucket.
/// 5. Returns the GCS URL of the log file.
pub async fn run_and_log_research(
    request: agentloop::types::research_request::ResearchRequest,
    gcs_client: &crate::services::gcs::gcs_client::GCSClient,
    execution_id: uuid::Uuid,
) -> std::result::Result<std::string::String, std::string::String> {
   // a. Call `create_agentloop_state()` to get a fresh `AppState` for the task.
   let agentloop_state = create_agentloop_state().await?;

   // b. Create a new session for the research task.
   // The `create_session` function is assumed to take the state and request, returning a session ID.
   // This allows the research goal to be passed separately to the sync loop, per instructions.
    // Create a session config from the request.
    let session_config = agentloop::types::session_config::SessionConfig {
        initial_instruction: Some(request.instruction.clone()),
        ..agentloop::types::session_config::SessionConfig::default()
    };

    let session_id = session_manager::create_session(
        agentloop_state.clone(),
        session_config,
    )
    .await;

    // c. Construct the callback and call the synchronous research loop.
    let progress_callback: agentloop::types::progress_update::ProgressCallback =
        Box::new(move |update: agentloop::types::progress_update::ProgressUpdate| {
            Box::pin(async move {
                // Progress logging removed - no database interaction
                let _ = (execution_id, update);
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        });

    let conversation_history = run_research_loop_sync(
        agentloop_state,
        session_id,
        request.instruction,
        Some(progress_callback),
    )
    .await
    .map_err(|e| format!("Failed to run research task: {e}"))?;

    // d. If the call is successful, serialize the resulting `Vec<crate::types::conversation_entry::ConversationEntry>` to a JSON string.
    let history_json = match serde_json::to_string_pretty(&conversation_history) {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to serialize conversation history: {e}")),
    };

    // e. Upload the JSON string to GCS.
    let bucket_name = "bounti_prod_narrativ_private";
    let object_name = format!("infinite-research-logs/{execution_id}.json");
    let content_type = "application/json";

    match gcs_client
        .upload_raw_bytes(
            bucket_name,
            &object_name,
            content_type,
            history_json.into_bytes(),
            true, // disable_cache
            crate::services::gcs::gcs_operations::UrlFormat::GsProtocol, // Use gs:// format for logs
        )
        .await
    {
        Ok(gcs_url) => {
            // f. Return the GCS URL of the uploaded file.
            Ok(gcs_url)
        }
        Err(e) => Err(format!("Failed to upload research log to GCS: {e}")),
    }
}
