//! Declares and organizes modules related to dashboard routes.
//!
//! This module serves as an aggregator for dashboard functionalities,
//! including usage statistics and route configurations. Each primary
//! item, like the route configuration function, is in its own file
//! adhering to the project's coding standards.

pub mod usage_statistics;
pub mod chart_models;
pub mod configure_dashboard_routes; // New module declaration
pub mod daily_activity_stats;
pub mod kpi_summary;
pub mod series_data_point;
pub mod custom_events_stats;
pub mod baseline_count;
pub mod user_cohort_analysis;

// The configure_dashboard_routes function is now in its own file/module.
