//! Configures dashboard-specific routes for the Actix web application.
//!
//! This function defines the `/dashboard` scope and registers
//! all sub-routes related to dashboard functionalities. It is intended
//! to be called during the main application route setup.
//! Adheres to one-item-per-file and fully-qualified path guidelines.

// Fully qualified paths are used as per coding standards.

use actix_web::web;

/// Configures dashboard-specific routes.
///
/// Adds the /dashboard scope and its sub-routes to the Actix web application.
/// This function specifically registers the `get_usage_statistics` handler from
/// the `usage_statistics` module.
///
/// # Arguments
///
/// * `cfg` - A mutable reference to `actix_web::web::ServiceConfig`.
pub fn configure_dashboard_routes(cfg: &mut actix_web::web::ServiceConfig) {
   cfg.service(
       web::scope("")
           .service(crate::routes::dashboard::usage_statistics::get_usage_statistics)
           .service(crate::routes::dashboard::daily_activity_stats::get_daily_activity_stats)
            .service(crate::routes::dashboard::kpi_summary::get_kpi_summary)
           .service(crate::routes::dashboard::custom_events_stats::get_custom_events_stats)
           .service(crate::routes::dashboard::custom_events_stats::get_custom_events_daily_stats)
           .service(crate::routes::dashboard::custom_events_stats::get_cohort_registration_analysis)
           .service(crate::routes::dashboard::baseline_count::get_baseline_count)
           .service(crate::routes::dashboard::user_cohort_analysis::get_user_cohort_analysis_handler)
   );
}

// No tests in this file as it's for route configuration.
// Route functionality is typically tested via integration tests.
