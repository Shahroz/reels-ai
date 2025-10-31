//! Configures internal-only routes.

use actix_web::web;
use super::{run_infinite_research, run_one_time_research};

/// Sets up endpoints for internal services.
pub fn configure_internal_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(run_infinite_research::run_infinite_research)
        .service(run_one_time_research::run_one_time_research);
}
