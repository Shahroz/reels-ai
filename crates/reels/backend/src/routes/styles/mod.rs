//! Module organizing all style-related API route handlers and supporting types.
//!
//! This module follows the one-item-per-file structure. Each file defines a
//! single struct, function, or related implementation, promoting modularity.
//! It exports all necessary components for integrating style routes into the application.

pub mod configure_styles_routes;
pub mod create_style;
pub mod create_style_request;
pub mod create_style_from_creative;
pub mod create_style_from_creative_request;
pub mod delete_style;
pub mod get_style_by_id;
pub mod list_styles;
pub mod responses;
pub mod update_style;
pub mod update_style_request;
pub mod validation;
pub mod fetching;
pub mod storage;