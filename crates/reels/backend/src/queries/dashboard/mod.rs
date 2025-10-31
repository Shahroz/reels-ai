//! Central module for dashboard-related database queries.
//!
//! This module organizes sub-modules responsible for fetching data
//! for various dashboard components, such as KPI summaries, daily activity charts,
//! and usage statistics tables.
//! Adheres to one-item-per-file and FQN guidelines.
//!
//! Revision History
//! - 2025-06-18T17:52:23Z @USER: Created dashboard queries module.

pub mod query_daily_activity;
pub mod query_kpi_metrics;
pub mod query_usage_statistics;
pub mod query_custom_events_stats;
pub mod query_custom_events_daily_stats;
pub mod query_cohort_registration_analysis;
pub mod query_baseline_count;
pub mod query_user_cohort_analysis;