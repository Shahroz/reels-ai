//! Assets routes module.
//!
//! This module contains all HTTP route handlers for asset-related endpoints,
//! including CRUD operations and specialized functionality.

pub mod configure_assets_routes;
pub mod create_asset;
pub mod create_asset_request;
pub mod delete_asset;
pub mod get_asset_by_id;
pub mod get_upload_url;
pub mod get_upload_url_request;
pub mod get_upload_url_response;
pub mod confirm_upload;
pub mod confirm_upload_request;
pub mod confirm_upload_response;
pub mod list_assets;
pub mod patch_asset;
pub mod responses;
pub mod upload_validation;
pub mod validation;
pub mod error_response;
pub mod attach_assets;
pub mod detach_assets;
pub mod enhance_asset;
pub mod enhance_asset_error;
pub mod enhance_asset_error_from;
pub mod derive_short_label_from_prompt;
pub mod get_root_asset_name;
pub mod count_existing_name_variants;
pub mod compute_final_names;
pub mod validate_and_fetch_assets;
pub mod prepare_enhanced_assets_data;
pub mod extract_request_context_for_enhancement;
pub mod enhance_asset_request;
pub mod enhance_asset_response;
pub mod gcs_uri_extractor;
pub mod save_assets_from_gcs;
pub mod studio_graph;
pub mod quick_enhance_image;
pub mod quick_enhance_image_request;
pub mod quick_enhance_image_response;

