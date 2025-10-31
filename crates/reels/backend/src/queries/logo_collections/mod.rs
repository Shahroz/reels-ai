//! Defines the module for logo collections database query operations.
//!
//! This module centralizes all database queries for the `logo_collections` table,
//! separating database logic from API handler logic.
//! Follows the existing patterns for collections with logo-specific functionality.

pub mod create_logo_collection;
pub mod get_logo_collection_by_id;
pub mod list_logo_collections;
pub mod update_logo_collection;
pub mod delete_logo_collection;
pub mod add_asset_to_logo_collection;
pub mod remove_asset_from_logo_collection;
pub mod get_logo_collection_with_assets;
