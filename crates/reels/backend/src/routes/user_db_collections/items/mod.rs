//! User DB Collection Items routes module.
//!
//! This module declares the sub-modules for managing items within user-defined database collections.
//! It includes handlers for creating, listing, retrieving, updating, and deleting items.
//! Adheres to the 'one item per file' and FQN guidelines.

pub mod configure_user_db_collection_items_routes;
pub mod create_user_db_collection_item;
pub mod delete_user_db_collection_item;
pub mod get_user_db_collection_item;
pub mod list_user_db_collection_items;
pub mod query_user_db_collection_items;
pub mod update_user_db_collection_item;
pub mod query_user_db_collection_items_request; // Ensure this is present
pub mod query_user_db_collection_items_response;
pub mod user_db_collection_item_request;
