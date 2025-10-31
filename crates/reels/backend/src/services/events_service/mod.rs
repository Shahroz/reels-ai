//! Business events tracking service.
//!
//! This module contains custom event tracking for specific business actions.
//! Events are manually triggered by business logic and contain rich contextual data.
//! All events follow the same base structure for analytics consistency.
//! Request context in request_details, event-specific data in custom_details.
//!
//! This module is only compiled when the "events" feature is enabled.

#[cfg(feature = "events")]
pub mod vocal_tour_events;
#[cfg(feature = "events")]
pub mod request_context;
#[cfg(feature = "events")]
pub mod event_helpers;
#[cfg(feature = "events")]
pub mod auth_events;
#[cfg(feature = "events")]
pub mod asset_events;
#[cfg(feature = "events")]
pub mod studio_events;

// Re-exports for convenient access
#[cfg(feature = "events")]
pub use vocal_tour_events::*;
#[cfg(feature = "events")]
pub use request_context::*;
#[cfg(feature = "events")]
pub use event_helpers::*;
#[cfg(feature = "events")]
pub use auth_events::*;
#[cfg(feature = "events")]
pub use asset_events::*;
#[cfg(feature = "events")]
pub use studio_events::*; 