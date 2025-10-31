//! Initializes the application state, including merging internal and external tool configurations.
pub async fn init_app_state(
    config: crate::config::app_config::AppConfig,
    tool_schemas: Option<crate::tools::tools_schema::ToolsSchema>,
    tool_handler: Option<crate::tools::tool_handler::ToolHandler>,
) -> std::result::Result<crate::state::app_state::AppState, anyhow::Error> {
    // Initialize application state, passing external tool configs and the merged handlers.
    let app_state = crate::state::app_state::AppState::new(
        config,
        tool_schemas,
        tool_handler,
    );
    std::result::Result::Ok(app_state)
}
