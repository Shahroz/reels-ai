//! Analytics API routes for cohort funnel analysis.
//!
//! This module contains REST API endpoints for analytics functionality.
//! Organized following one-function-per-file Rust coding guidelines.
//! Provides admin-only access to comprehensive analytics features.
//! Enables cohort selection and funnel analysis for user behavior tracking.

pub mod get_cohort_funnel_analysis;
pub mod get_available_cohorts;
pub mod track_mobile_event;
pub mod track_mobile_event_request;

// Note: Functions are used directly in configure_analytics_routes

/// Configure analytics routes with proper middleware and authentication
pub fn configure_analytics_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/api/analytics")
            .wrap(crate::middleware::auth::JwtMiddleware)
            .service(get_cohort_funnel_analysis::get_cohort_funnel_analysis)
            .service(get_available_cohorts::get_available_cohorts)
            .service(get_available_cohorts::get_cohort_by_date)
            .service(track_mobile_event::track_mobile_event)
    );
} 