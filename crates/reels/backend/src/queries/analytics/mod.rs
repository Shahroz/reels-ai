//! Analytics database queries for cohort funnel analysis.
//!
//! This module contains all database query functions for analytics tracking.
//! Organized following one-function-per-file Rust coding guidelines.
//! Provides comprehensive support for cohort selection and funnel analysis.
//! Enables detailed tracking and analysis of user behavior patterns.

pub mod get_cohort_funnel_analysis;
pub mod get_available_cohorts;
pub mod insert_analytics_event;

// Re-exports for convenient access
pub use get_cohort_funnel_analysis::{
    get_cohort_funnel_analysis, 
    CohortFunnelAnalysisParams, 
    CohortFunnelAnalysisResult, 
    FunnelStepResult
};
pub use get_available_cohorts::{
    get_available_cohorts, 
    get_cohort_by_date, 
    AvailableCohort
};
pub use insert_analytics_event::{
    insert_analytics_event,
    insert_analytics_event_returning,
    insert_analytics_events_batch, 
    get_analytics_event_by_id
}; 