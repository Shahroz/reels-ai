//! Analytics services for business logic coordination.
//!
//! This module contains service layer functions for analytics operations.
//! Organized following one-item-per-file Rust coding guidelines.
//! Provides high-level business logic and validation for analytics features.
//! Coordinates between database queries and API endpoints.

pub mod cohort_funnel_service;
pub mod analytics_event_service;

// Re-exports for convenient access
pub use cohort_funnel_service::{CohortFunnelService, ServiceError as CohortServiceError};
pub use analytics_event_service::{AnalyticsEventService, EventServiceError}; 