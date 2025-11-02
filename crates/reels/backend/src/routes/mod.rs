// backend/src/routes/mod.rs

// All route folders have been deleted. Only AgentLoop routes remain active.
use agentloop::config::routes::{configure_routes as configure_loupe_routes};
use actix_web::web;

pub mod error_response;
pub mod health;
pub mod storage;

/// Configures all API routes for the application, including AgentLoop integration.
/// NOTE: All route folders have been deleted. Only AgentLoop routes remain active.
pub fn config(cfg: &mut web::ServiceConfig) {
    // Health check endpoint - must be first for easy health checks
    cfg.route("/health", web::get().to(health::health_check));
    
    // Storage routes for serving local video files
    storage::configure_storage_routes(cfg);
    
    // Mount AgentLoop routes under /loupe, passing in the pre-initialized AppState
    cfg.service(
        web::scope("/loupe") // Add scope for agentloop service
            .configure(configure_loupe_routes),
    );
}
