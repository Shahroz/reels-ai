//! Admin routes for managing unlimited access grants.
//!
//! This module provides admin endpoints for granting, revoking, and
//! listing unlimited credit access grants for users and organizations.

pub mod grant_unlimited_to_user_handler;
pub mod grant_unlimited_to_user_request;
pub mod grant_unlimited_to_user_response;
pub mod revoke_unlimited_from_user_handler;
pub mod revoke_unlimited_from_user_request;
pub mod revoke_unlimited_from_user_response;
pub mod list_unlimited_grants_handler;
pub mod list_unlimited_grants_params;
pub mod list_unlimited_grants_response;
pub mod configure_unlimited_access_routes;

