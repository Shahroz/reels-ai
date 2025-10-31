//! Module for organization-related API endpoints.
//!
//! This module groups handlers and configuration for managing organizations,
//! such as creating new organizations and listing organizations for a user.
//! It follows the project's structure and coding guidelines.

pub mod configure_organization_routes;
pub mod create_organization_handler;
pub mod create_organization_request;
pub mod get_organization_handler;
pub mod list_organizations_handler;
pub mod update_organization_handler;
pub mod update_organization_request;
pub mod delete_organization_handler;
pub mod member_response;
pub mod list_members_handler;
pub mod get_organization_members_for_credits;
pub mod remove_member_handler;
pub mod invite_member_handler;
pub mod list_sent_invitations_handler;
