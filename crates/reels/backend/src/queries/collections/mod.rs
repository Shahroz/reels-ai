//! Defines the module for collections-related database query operations.
//!
//! This module centralizes all database queries for the `collections` table,
//! separating database logic from API handler logic.
//! Adheres to one-item-per-file and FQN guidelines.

pub mod count_collections;
pub mod create_collection;
pub mod delete_collection;
pub mod get_collection_by_id;
pub mod get_collection_hierarchy;
pub mod get_collection_with_assets;
pub mod get_collection_with_sharing;
pub mod list_collections;
pub mod list_collections_with_sharing;
pub mod list_collections_with_permissions;
pub mod update_collection;
pub mod update_collection_organization_id;