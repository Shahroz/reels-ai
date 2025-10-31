//! Orchestrates the initial setup and configuration of the application state.
//!
//! This module provides the `configure_app` function which loads environment
//! configuration and initializes the core application state (`AppState`).
//! It acts as the central point for bootstrapping the application's shared context.
//! Adheres strictly to the project's Rust coding standards.


/// Loads configuration and initializes the application state.
///
/// This function performs the following steps:
/// 1. Calls `crate::config::load_env_config::load_env_config` to load configuration
///    from environment variables (potentially using a `.env` file).
/// 2. Uses the loaded configuration to call `crate::state::init_app_state::init_app_state`,
///    which constructs the `AppState`.
///
/// Returns the initialized `AppState` wrapped in a `Result`, or an error
/// if either configuration loading or state initialization fails.
pub async fn configure_app(
    tool_schemas: Option<crate::tools::tools_schema::ToolsSchema>,
    tool_handler: Option<crate::tools::tool_handler::ToolHandler>,
) -> std::result::Result<
    crate::state::app_state::AppState,
    std::boxed::Box<dyn std::error::Error>,
> {
    // 1. Load configuration from the environment.
    // Use fully qualified path for the function call.
    let config = crate::config::load_env_config::load_env_config()?;

    // 2. Initialize the application state using the loaded configuration.
    // Use fully qualified path for the function call.
    let app_state = crate::state::init_app_state::init_app_state(config, tool_schemas, tool_handler).await?;

    // 3. Return the successfully initialized state.
    // Use fully qualified path for Ok variant.
    std::result::Result::Ok(app_state)
}

// No tests are defined here as this function primarily orchestrates calls
// to other units (`load_env_config`, `init_app_state`) which should have their own tests.
// Integration tests covering the application startup would implicitly validate this flow.