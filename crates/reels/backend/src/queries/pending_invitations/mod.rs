//! Defines the module for pending invitation-related database queries.
//!
//! This module is responsible for creating, retrieving, and deleting
//! invitations for users to join organizations.
//! It manages the lifecycle of an invitation.
//! Adheres to one-item-per-file and FQN guidelines.
//!
//! Revision History
//! - 2025-06-18T12:55:45Z @AI: Created pending_invitations query module.

pub mod create_pending_invitation;
pub mod delete_pending_invitation;
pub mod find_pending_invitation_by_org_and_email;
pub mod find_pending_invitation_by_token;
pub mod find_pending_invitations_for_email;
pub mod find_pending_invitations_for_organization;
