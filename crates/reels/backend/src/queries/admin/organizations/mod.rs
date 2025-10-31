//! Admin organization query operations.
//!
//! This module exposes admin-level queries for managing organizations. These queries
//! allow admins to list all organizations, batch-add members, and update organization
//! ownership across the entire platform.

pub mod batch_add_members;
pub mod list_all_organizations_admin;
pub mod list_organizations_with_credits;
pub mod enriched_organization_with_credits;
pub mod update_organization_owner;
pub mod services;

pub use batch_add_members::batch_add_members;
pub use list_all_organizations_admin::list_all_organizations_admin;
pub use list_organizations_with_credits::list_organizations_with_credits;
pub use enriched_organization_with_credits::EnrichedOrganizationWithCredits;
pub use update_organization_owner::update_organization_owner;
