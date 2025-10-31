//! Module aggregating API Key management route handlers and configuration.
//!
//! This module organizes the individual components related to API key endpoints:
//! - The response structure for key creation.
//! - Handlers for POST (create), GET (list), and DELETE (revoke) operations.
//! - The service configuration function to register these handlers.
//! It adheres to the one-item-per-file standard by declaring sub-modules.

pub mod configure_api_key_routes;
pub mod create_api_key_request;
pub mod create_api_key_response;
pub mod create_key_handler;
pub mod delete_key_handler;
pub mod list_keys_handler;
pub mod list_keys_params;
pub mod list_keys_response;
