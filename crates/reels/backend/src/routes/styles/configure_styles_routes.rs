//! Configures the Actix Web service for all style-related routes.
//!
//! This function groups all style API endpoints (`list`, `get`, `create`, `update`, `delete`)
//! under the `/api/styles` scope within the Actix Web application configuration.
//! It ensures that all handlers defined in this module are registered correctly.
//! Called during application setup.

use crate::middleware::credits_guard::require_generate_style;

/// Configures the routes for style management under `/api/styles`.
pub fn configure_styles_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("") // Base path is already /api/styles from the caller
            .service(crate::routes::styles::list_styles::list_styles)
            .service(crate::routes::styles::get_style_by_id::get_style_by_id)
            .service(crate::routes::styles::update_style::update_style)
            .service(crate::routes::styles::delete_style::delete_style)
            // Style creation endpoints require credits
            .service(
                actix_web::web::scope("")
                    .wrap(require_generate_style())
                    .service(crate::routes::styles::create_style::create_style)
                    .service(crate::routes::styles::create_style_from_creative::create_style_from_creative)
            ),
    );
}
