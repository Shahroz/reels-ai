//! Defines parameter structures for Narrativ-hosted agent tools.
//!
//! Each file in this module defines the parameters for a specific tool,
//! adhering to the one-item-per-file guideline. These structures are used
//! for strong typing within Narrativ's tool handlers and for generating
//! JSON schemas for AgentLoop.

pub mod browse_raw_params;
pub mod browse_with_query_params;
pub mod google_search_browse_params;
pub mod save_context_params;
pub mod search_params;
pub mod narrativ_document_count_params;
pub mod narrativ_document_delete_params;
pub mod narrativ_document_fetch_list_params;
pub mod narrativ_document_find_by_id_params;
pub mod narrativ_document_insert_params;
pub mod narrativ_document_update_params;
pub mod create_user_db_collection_item_params;
pub mod create_user_db_collection_params;
pub mod delete_user_db_collection_item_params;
pub mod delete_user_db_collection_params;
pub mod get_user_db_collection_item_params;
pub mod get_user_db_collection_params;
pub mod list_user_db_collection_items_tool_params;
pub mod list_user_db_collections_params;
pub mod query_user_db_collection_items_params;
pub mod update_user_db_collection_item_params;
pub mod update_user_db_collection_params;
pub mod update_user_db_collection_schema_params;

pub mod generate_creative_from_bundle_params;
pub mod generate_creative_params;
pub mod generate_style_from_url_params;
pub mod list_assets_params;
pub mod save_asset_params;
pub mod list_bundles_params;
pub mod list_collections_params;
pub mod create_collection_params;
pub mod list_formats_params;
pub mod list_styles_params;
pub mod property_research_params;
pub mod property_description_to_contents_params;
pub mod retouch_images_params;
pub mod vocal_tour_params;
pub mod quick_enhance_image_params;
pub mod generate_reel_params;
