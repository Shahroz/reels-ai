//! Configures all One-Time Research-related routes.
//!
//! Mounted under /api/one-time-researches with JWT authentication.

use actix_web::web;

use super::{
    create_one_time_research, delete_one_time_research, get_one_time_research,
    list_one_time_researches,
};

/// Sets up endpoints for One-Time Research operations within the /api/one-time-researches scope.
pub fn configure_one_time_researches_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_one_time_researches::list_one_time_researches)
        .service(get_one_time_research::get_one_time_research)
        .service(create_one_time_research::create_one_time_research)
        .service(delete_one_time_research::delete_one_time_research);
}