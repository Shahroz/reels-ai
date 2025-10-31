//! Defines handlers for Narrativ-hosted agent tools.
//!
//! Each file in this module implements the logic for a specific tool,
//! taking strongly-typed parameters and returning results compatible with
//! AgentLoop's `FullToolResponse` and `UserToolResponse`.

pub mod handle_narrativ_browse_raw;
pub mod handle_narrativ_browse_with_query;
pub mod handle_narrativ_save_context;
pub mod handle_narrativ_search;
pub mod handle_google_search_browse;
pub mod handle_list_assets;
pub mod handle_save_asset;
pub mod handle_list_bundles;
pub mod handle_list_collections;
pub mod handle_create_collection;
pub mod handle_list_formats;
pub mod handle_list_styles;

// New User DB Collection and Item handlers
pub mod handle_create_user_db_collection;
pub mod handle_delete_user_db_collection;
pub mod handle_get_user_db_collection;
pub mod handle_list_user_db_collections;
pub mod handle_update_user_db_collection;
pub mod handle_update_user_db_collection_schema;
pub mod handle_create_user_db_collection_item;
pub mod handle_delete_user_db_collection_item;
pub mod handle_get_user_db_collection_item;
pub mod handle_list_user_db_collection_items_tool;
pub mod handle_query_user_db_collection_items;
pub mod handle_update_user_db_collection_item;
pub mod handle_narrativ_document_delete;
pub mod handle_narrativ_document_fetch_list;
pub mod handle_narrativ_document_find_by_id;
pub mod handle_narrativ_document_insert;
pub mod handle_narrativ_document_update;
pub mod handle_narrativ_document_count;

pub mod handle_generate_creative;
pub mod handle_generate_creative_from_bundle;
pub mod handle_generate_style_from_url;
pub mod handle_property_research;
pub mod handle_property_description_to_contents;
pub mod handle_retouch_images;
pub mod handle_vocal_tour;
pub mod handle_quick_enhance_image;
pub mod handle_generate_reel;
