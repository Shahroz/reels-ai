//! User DB Collections routes module.
//!
//! This module declares the sub-modules for managing user-defined database collections.
//! It includes handlers for creating, listing, retrieving, updating, and deleting collections.
//! Adheres to the 'one item per file' and FQN guidelines.

pub mod configure_user_db_collections_routes;
pub mod create_user_db_collection;
pub mod create_user_db_collection_request;
pub mod delete_user_db_collection;
pub mod get_user_db_collection;
pub mod list_user_db_collections;
pub mod update_user_db_collection;
pub mod update_user_db_collection_request;
pub mod items; // Added for item-specific routes
pub mod update_user_db_collection_schema;
pub mod update_user_db_collection_schema_request;
pub mod copy_predefined_collection;
pub mod get_or_create_user_collection_by_predefined;
pub mod get_or_create_user_collection_by_predefined_name;
pub mod user_collection_service;