//! Custom analytics events database models and types.
//!
//! This module contains all data structures for custom analytics event tracking.
//! Organized following one-item-per-file Rust coding guidelines.
//! Provides comprehensive support for business logic event tracking.
//! Enables cohort-based funnel analysis with detailed event metadata.

pub mod event_source; // Deprecated - kept for backwards compatibility during transition
pub mod custom_event_category;
pub mod analytics_event;
pub mod new_analytics_event;

// Re-exports for convenient access
// pub use event_source::EventSource; // REMOVED - no longer used with custom-only events
pub use custom_event_category::CustomEventCategory;
pub use analytics_event::AnalyticsEvent;
pub use new_analytics_event::NewAnalyticsEvent; 