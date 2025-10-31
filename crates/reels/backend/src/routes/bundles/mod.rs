//! Organizes all route handlers and request/response structures related to bundles.
//!
//! This module serves as the entry point for bundle-related API endpoints,
//! declaring sub-modules for each specific handler (create, list, get, update, delete)
//! and any associated request or response data structures.

pub mod configure_bundle_routes;
pub mod create_bundle_handler;
pub mod create_bundle_request;
pub mod update_bundle_request;
pub mod delete_bundle_handler; // Referenced in configure_bundle_routes.rs
pub mod get_bundle_handler;    // Referenced in configure_bundle_routes.rs
pub mod list_bundles_handler;  // Referenced in configure_bundle_routes.rs
pub mod update_bundle_handler; // Referenced in configure_bundle_routes.rs

// New response struct for listing bundles
pub mod list_bundles_response;
pub use list_bundles_response::ListExpandedBundlesResponse;
// Add other bundle-related sub-modules here as they are created,
// for example, request/response structs for update operations if separate.