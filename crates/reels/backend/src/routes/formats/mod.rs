//! Module for Creative Format endpoints.
//!
//! Defines routing and handlers for managing public and custom creative formats.

// Public format handler
pub mod create_custom_creative_format;
pub mod create_custom_format_request;
pub mod copy_custom_creative_format;
pub mod delete_custom_creative_format;
pub mod list_custom_creative_formats;
pub mod update_custom_creative_format;

// Route configuration
pub mod configure_formats_routes;
