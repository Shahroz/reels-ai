//! Module for internal-only routes not intended for direct user consumption.
//!
//! These routes are typically called by other services, like Google Cloud Scheduler,
//! and have their own authentication mechanisms instead of the standard user JWT flow.

pub mod configure_internal_routes;
pub mod run_infinite_research;
pub mod run_one_time_research;
