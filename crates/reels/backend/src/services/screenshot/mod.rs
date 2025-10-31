//! Screenshot service module organization and re-exports.
//!
//! This module organizes screenshot functionality into separate files following
//! the one-item-per-file principle. Each component (trait, implementations, factory)
//! is defined in its own file and re-exported here for convenient access.
//! The module supports both production and test environments through dependency
//! injection patterns that eliminate environment-specific conditional logic.

pub mod screenshot_service;
pub mod screenshot_result;
pub mod screenshot_config;
pub mod zyte_screenshot_service;
pub mod mock_screenshot_service;
pub mod service_factory;