//! Module for queries related to user DB collections.
//!
//! Contains sub-modules for queries on collections themselves and their items.
//! Adheres to 'one item per file' and FQN guidelines.

pub mod items;
pub mod create_user_db_collection_query;
pub mod delete_user_db_collection_query;
pub mod get_user_db_collection_query;
pub mod list_user_db_collections_query;
pub mod update_user_db_collection_query;
pub mod update_user_db_collection_schema_query;
// Potentially: pub mod get_user_db_collection_query; etc.
