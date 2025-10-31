//! Configures all Infinite Research-related routes.

use actix_web::web;
use super::{
    create_infinite_research, delete_infinite_research, get_execution_log, get_infinite_research,
    list_infinite_research_executions, list_infinite_researches, update_infinite_research,
    update_infinite_research_status, run_infinite_research_manually,
};

/// Sets up endpoints for Infinite Research operations.
pub fn configure_infinite_researches_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_infinite_research::create_infinite_research)
        .service(list_infinite_researches::list_infinite_researches)
        .service(get_infinite_research::get_infinite_research)
        .service(update_infinite_research::update_infinite_research)
        .service(delete_infinite_research::delete_infinite_research)
        .service(list_infinite_research_executions::list_infinite_research_executions)
        .service(update_infinite_research_status::update_infinite_research_status)
        .service(get_execution_log::get_execution_log)
        .service(run_infinite_research_manually::run_infinite_research_manually);
}
