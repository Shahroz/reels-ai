//! Admin routes for organization management.
//!
//! This module contains HTTP handlers for admin-only organization operations including
//! listing all organizations, creating organizations with specified owners, updating
//! organization details and ownership, and batch adding members.

pub mod admin_add_members_handler;
pub mod admin_add_members_request;
pub mod admin_add_members_response;
pub mod admin_create_organization_handler;
pub mod admin_create_organization_request;
pub mod admin_list_members_handler;
pub mod admin_update_organization_handler;
pub mod admin_update_organization_request;
pub mod configure_admin_organizations_routes;
pub mod list_all_organizations_handler;
pub mod list_all_organizations_params;
pub mod list_all_organizations_response;
pub mod update_organization_credits_handler;
pub mod update_organization_credits_request;
pub mod update_organization_credits_response;
