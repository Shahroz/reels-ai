//! Organization credit allocation query functions.
//!
//! This module contains database query functions for organization credit allocations.
//! Each function is isolated in its own file following the one-file-per-item pattern.

pub mod admin_update_organization_credits_with_transaction;
pub mod create_organization_credit_allocation;
pub mod get_organization_credit_allocation_by_org_id;
pub mod update_organization_credit_allocation;
pub mod deduct_organization_credits;
pub mod deduct_organization_credits_with_transaction;
pub mod create_or_update_organization_credit_allocation;

