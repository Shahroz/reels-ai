//! Photo extraction services for processing video frames and images.

pub mod extract_frames_from_video_on_gcs;

// Shared types and utilities
pub mod output_format;
pub mod conversion_result;
pub mod check_imagemagick_format_support;
pub mod convert_raw_image_on_gcs;

// Format-specific conversion functions
pub mod convert_heic_on_gcs;
pub mod convert_dng_on_gcs; 