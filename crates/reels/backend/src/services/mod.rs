// backend/src/services/mod.rs

//! Services module for the application.
//!
//! This module contains all the business logic services used by the application.
//! Each service is responsible for a specific domain of functionality.
//!
//! Only agent-related services are kept.

pub mod agent_service;
pub mod http_request;
pub mod screenshot;
pub mod gcs;
