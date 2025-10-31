// backend/src/services/mod.rs

//! Services module for the application.
//!
//! This module contains all the business logic services used by the application.
//! Each service is responsible for a specific domain of functionality.

pub mod trial_service;
pub mod agent_service;
pub mod pdf_conversion_service;
pub mod http_request;
pub mod gennodes;
pub mod creative_generation_service;
pub mod photo_extraction;
pub mod screenshot;
pub mod google_drive;
pub mod gcs;
pub mod encryption;
pub mod content_extraction;
pub mod metadata_extraction;
pub mod billing;
pub mod stripe_webhook_handler;
pub mod imageboard_webhook_handler;
pub use imageboard_webhook_handler::imageboard_webhook_service;
pub use imageboard_webhook_handler::handlers;
#[cfg(feature = "events")]
pub mod analytics;
pub mod session_manager;
#[cfg(feature = "events")]
pub mod events_service;
pub mod dub;
pub mod watermarking;
pub mod content_generation_functions;
pub mod permission_resolver;
pub mod imageboard_client;
pub mod email_service;
pub mod credit_rewards;
pub mod credits_service;
