//! Defines query functions for share-related operations.
//!
//! This module centralizes all database interactions related to object shares,
//! abstracting the query logic away from the route handlers.
//! Adheres to one-item-per-file and FQN guidelines.

pub mod batch_permission_check;
pub mod can_user_manage_object_shares;
pub mod check_object_ownership;
pub mod check_user_has_editor_share;
pub mod delete_share_by_id;
pub mod find_share_by_id;
pub mod find_shares;
pub mod find_user_id_by_email;
pub mod inherit_shares_from_asset;
pub mod upsert_share;
pub mod validate_object_ownership;