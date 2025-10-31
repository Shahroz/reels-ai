//! Module for Creative endpoints.
//!
//! Defines routing and handlers for CREATIVE resources.

pub mod configure_creatives_routes;
pub mod create_creative;
pub mod create_creative_request;
pub mod creative_asset_utils;
pub mod delete_creative;
pub mod discard_draft;
pub mod duplicate_creative;
pub mod edit_creative;
pub mod edit_creative_request;
pub mod generate_creative;
pub mod generate_creative_request;
pub mod generate_creative_from_bundle_handler; // Added
pub mod generate_creative_from_bundle_request; // Added
pub mod responses;
pub mod save_creative_as_style;
pub mod text_rewrite;
pub mod text_rewrite_request;
// Add the new module for the typed LLM response
pub mod text_rewrite_response;
pub mod get_creative_content_response;
pub mod get_creative_content;
pub mod get_creative_by_id;
pub mod list_creatives;
pub mod publish_draft;
pub mod update_creative_name;
pub mod update_creative_name_request;
