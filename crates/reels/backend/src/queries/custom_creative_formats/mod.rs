//! Module for custom_creative_format-related database query functions.
//!
//! This module aggregates specific query operations for the `custom_creative_formats` entity.
//! It adheres to the one-item-per-file and FQN guidelines.
//! Organizes functions for managing custom creative formats.

pub mod count;
pub mod create;
pub mod delete;
pub mod exists;
pub mod find_for_update;
pub mod find_one_for_copy;
pub mod get_organization_id;
pub mod insert_copy;
pub mod list;
pub mod share_with_org;
pub mod update;
