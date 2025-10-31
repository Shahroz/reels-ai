//! Provides the public setup function for initializing the AgentLoop core application state.
//!
//! This module allows host applications to configure and initialize AgentLoop,
//! including injecting their own custom tool definitions and handlers.

// Use fully qualified paths as per guidelines
// use crate::config::app_config::AppConfig; - Not needed directly, AppConfig is a type alias target
// use crate::state::app_state::AppState; - Not needed directly, AppState is a type alias target
// use crate::types::tool_definition::ToolDefinition; - Not needed directly, ToolDefinition is a type alias target
// use crate::tools::tool_handler::ToolHandler; - Not needed directly, ToolHandler is a type alias target
// use actix_web::web::Data; - Not needed directly, Data is a type alias target

/// Sets up the AgentLoop application state, allowing injection of external tools.
///
/// This function initializes the core components of AgentLoop, including its
/// configuration, internal tools, and any external tools provided by the host application.
/// It returns a shared `AppState` instance wrapped in `actix_web::web::Data` suitable
/// for use in an Actix-web application.
///
/// # Arguments
/// * `config` - The `crate::config::app_config::AppConfig` containing application settings.
/// * `external_tool_definitions` - A `Vec<crate::types::tool_definition::ToolDefinition>`
///   for custom tools provided by the host.
/// * `external_tool_handlers` - A `std::collections::HashMap<String, crate::tools::tool_handler::ToolHandler>`
///   mapping names to handler functions for custom tools.
///
/// # Returns
/// A `Result` containing the shared `actix_web::web::Data<crate::state::app_state::AppState>`,
/// or an error string if initialization fails.
pub async fn setup_agentloop_core(
    config: crate::config::app_config::AppConfig,
    tool_schemas: Option<crate::tools::tools_schema::ToolsSchema>,
    tool_handler: Option<crate::tools::tool_handler::ToolHandler>,
) -> Result<actix_web::web::Data<crate::state::app_state::AppState>, String> {
    match crate::state::init_app_state::init_app_state(
        config,
        tool_schemas,
        tool_handler
    ).await {
        Ok(app_state) => Ok(actix_web::web::Data::new(app_state)),
        Err(e) => Err(format!("Failed to initialize AgentLoop AppState: {}", e)),
    }
}