//! Module for queries related to items within user DB collections.
//!
//! Each file in this module typically corresponds to a specific query operation
//! for user DB collection items, such as listing, creating, or retrieving.
//! Adheres to 'one item per file' and FQN guidelines.

pub mod list_user_db_collection_items_query;
pub mod query_user_db_collection_items_query;
pub mod update_user_db_collection_item_query;

pub mod create_user_db_collection_item_query;
pub mod delete_user_db_collection_item_query;
pub mod get_user_db_collection_item_query;