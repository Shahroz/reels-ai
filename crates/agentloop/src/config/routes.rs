//! Configures Actix-web routes for the AgentLoop application.
//!
//! Maps HTTP endpoints to their corresponding handler functions,
//! ensuring that requests are routed correctly based on method and path.
//! Adheres to the one-item-per-file and FQN guidelines.
// Revision History
// - 2025-05-13T19:28:30Z @AI: Modify configure_routes to accept external tools and set up AppState.
// - 2025-05-12T12:49:39Z @AI: Add routes for get_session_state and load_session_state.
// - 2025-04-24T15:20:30Z @AI: Implement route configuration based on instruction.
// - 2025-04-24T12:45:12Z @AI: Initial placeholder implementation.

/// Configures the Actix-web application routes.
///
/// This function is typically called during application setup to register
/// all defined endpoints and link them to their handler logic.
///
/// # Arguments
/// * `cfg` - A mutable reference to the `ServiceConfig` used to register routes.
/// * `app_config` - The application's configuration.
/// * `tool_schemas` - Optional JSON schema definitions for external tools.
/// * `tool_handler` - Optional merged handler for external tools.
pub fn configure_routes(
    cfg: &mut actix_web::web::ServiceConfig,
    // app_config: crate::config::app_config::AppConfig, // Removed
    // tool_schemas: Option<serde_json::Value>, // Removed
    // tool_handler: Option<crate::tools::tool_handler::ToolHandler>, // Removed
) {
    // AppState is now expected to be set globally in main_internal.rs
    // and accessed by handlers via Data<AppState> extractor.
    // No longer creating or setting AppState here.

    // 5. Configure Swagger UI
    let openapi_spec = crate::openapi::api_doc();
    cfg.service(
        utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}")
            .url("/api-docs/openapi.json", openapi_spec),
    );

    // 6. Configure application routes
    // Note: Using fully qualified paths for handlers and actix_web types as per guidelines.
    cfg.route(
        "/research",
        actix_web::web::post().to(crate::handlers::start_research::start_research),
    );
    cfg.route(
        "/session/{session_id}/status",
        actix_web::web::get().to(crate::handlers::get_status::get_status),
    );
    cfg.route(
        "/session/{session_id}/message",
        actix_web::web::post().to(crate::handlers::post_message::post_message),
    );
    cfg.route(
        "/session/{session_id}/terminate",
        actix_web::web::post().to(crate::handlers::terminate_session::terminate_session),
    );
    cfg.route(
        "/session/{session_id}/stream", // WebSocket endpoint
        actix_web::web::get().to(crate::handlers::conversation_stream::conversation_stream),
    );
    cfg.route(
        "/session/{session_id}/state", // Get session state
        actix_web::web::get().to(crate::handlers::get_session_state::get_session_state),
    );
    cfg.route(
        "/session/load", // Load session state
        actix_web::web::post().to(crate::handlers::load_session_state::load_session_state),
    );
}

pub fn configure_internal(
    cfg: &mut actix_web::web::ServiceConfig,
) {
    // 5. Configure Swagger UI
    let openapi_spec = crate::openapi::api_doc();
    cfg.service(
        utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}")
            .url("/api-docs/openapi.json", openapi_spec),
    );

    // 6. Configure application routes
    // Note: Using fully qualified paths for handlers and actix_web types as per guidelines.
    cfg.route(
        "/research",
        actix_web::web::post().to(crate::handlers::start_research::start_research),
    );
    cfg.route(
        "/session/{session_id}/status",
        actix_web::web::get().to(crate::handlers::get_status::get_status),
    );
    cfg.route(
        "/session/{session_id}/message",
        actix_web::web::post().to(crate::handlers::post_message::post_message),
    );
    cfg.route(
        "/session/{session_id}/terminate",
        actix_web::web::post().to(crate::handlers::terminate_session::terminate_session),
    );
    cfg.route(
        "/session/{session_id}/stream", // WebSocket endpoint
        actix_web::web::get().to(crate::handlers::conversation_stream::conversation_stream),
    );
    cfg.route(
        "/session/{session_id}/state", // Get session state
        actix_web::web::get().to(crate::handlers::get_session_state::get_session_state),
    );
   cfg.route(
       "/session/load", // Load session state
       actix_web::web::post().to(crate::handlers::load_session_state::load_session_state),
   );
   cfg.route(
       "/research/run-sync",
       actix_web::web::post().to(crate::handlers::run_research_sync::run_research_sync),
   );
}

// No tests are typically included in the route configuration file itself.
// Integration tests would cover routing behavior.
