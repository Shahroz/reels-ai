//! Configuration for Studio routes.
//!
//! Sets up endpoints for Studio tutorial event tracking under `/api/studio`.

use actix_web::web;

/// Configure Studio routes
pub fn configure_studio_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(super::tutorial_events::log_tutorial_button_click)
    );
}
