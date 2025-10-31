//! Watermarking services for applying logos to assets.
//!
//! This module provides watermarking functionality including logo positioning,
//! image processing using photon-rs, and asset generation.
//! Each component is in its own file following the "code as a graph" philosophy.

// Error handling
pub mod watermark_error;

// Core processing logic  
pub mod photon_processor;
pub mod apply_batch_watermark_sync_photon;

// Validation functions
pub mod validate_watermark_request;
pub mod validate_image_assets;
pub mod validate_bytes_size;

// Asset operations
pub mod get_asset_by_id;
pub mod fetch_and_validate_source_asset;
pub mod download_asset_bytes;
pub mod download_and_validate_image;

// Watermark processing
pub mod process_watermarks;
pub mod process_single_watermark;

// Asset creation
pub mod create_final_watermarked_asset;
pub mod create_watermarked_asset;

// Utility functions
pub mod generate_watermarked_filename;
pub mod get_content_type_from_filename;

// Re-export main function for backwards compatibility
pub use apply_batch_watermark_sync_photon::apply_batch_watermark_sync_photon;
