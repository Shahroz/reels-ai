//! Photon-rs image processing service for watermarking operations.
//!
//! This module provides image processing capabilities using photon-rs,
//! replacing ImageMagick for enhanced security and performance.
//! Handles logo positioning, sizing, opacity, and composition operations.

use crate::schemas::watermark_schemas::{WatermarkConfig, WatermarkPosition, WatermarkSize, CornerPosition, EdgePosition};
use photon_rs::native::{open_image, save_image, open_image_from_bytes};
use photon_rs::PhotonImage;
use std::path::Path;

/// Error types for photon-rs processing operations
#[derive(Debug, thiserror::Error)]
pub enum PhotonError {
    #[error("Failed to open image: {0}")]
    ImageOpen(String),
    #[error("Failed to save image: {0}")]
    ImageSave(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Image processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Path validation failed: {0}")]
    InvalidPath(String),
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image processing error: {0}")]
    ImageProcessing(String),
}

/// Result type for photon operations
pub type PhotonResult<T> = Result<T, PhotonError>;

/// Photon-rs watermarking processor
pub struct PhotonProcessor;

impl PhotonProcessor {
    /// Creates a new PhotonProcessor instance
    pub fn new() -> Self {
        Self
    }

    /// Applies a watermark to an image using photon-rs (in-memory processing)
    pub async fn apply_watermark_from_bytes(
        &self,
        source_image_bytes: &[u8],
        logo_image_bytes: &[u8],
        config: &WatermarkConfig,
    ) -> PhotonResult<Vec<u8>> {
        log::info!("Starting photon-rs watermark application (in-memory)");
        log::info!("Config: {:?}", config);

        // Validate configuration
        self.validate_watermark_config(config)?;

        // Load images from bytes
        log::info!("Opening source image from bytes with photon-rs");
        let mut source_img = open_image_from_bytes(source_image_bytes)
            .map_err(|e| PhotonError::ImageOpen(format!("Failed to load source image from bytes: {}", e)))?;

        log::info!("Opening logo image from bytes with photon-rs");
        let mut logo_img = open_image_from_bytes(logo_image_bytes)
            .map_err(|e| PhotonError::ImageOpen(format!("Failed to load logo image from bytes: {}", e)))?;

        // Get image dimensions
        let source_width = source_img.get_width();
        let source_height = source_img.get_height();
        let logo_width = logo_img.get_width();
        let logo_height = logo_img.get_height();
        
        log::info!("Source image dimensions: {}x{}", source_width, source_height);
        log::info!("Logo image dimensions: {}x{}", logo_width, logo_height);

        // Resize logo according to configuration
        logo_img = self.resize_logo(&logo_img, &config.size, source_width, source_height)?;
        let resized_logo_width = logo_img.get_width();
        let resized_logo_height = logo_img.get_height();
        log::info!("Resized logo dimensions: {}x{}", resized_logo_width, resized_logo_height);

        // Apply opacity to logo
        if config.opacity < 1.0 {
            log::info!("Applying opacity {} to logo", config.opacity);
            self.apply_opacity(&mut logo_img, config.opacity as f64)?;
        }

        // Calculate position for logo placement
        let (x, y) = self.calculate_position(&config.position, source_width, source_height, resized_logo_width, resized_logo_height)?;
        log::info!("Calculated logo position: ({}, {})", x, y);

        // Composite logo onto source image
        log::info!("Compositing logo onto source image");
        self.composite_images(&mut source_img, &logo_img, x, y)?;

        // Convert result to PNG bytes
        log::info!("Converting result to PNG bytes");
        let result_bytes = self.photon_image_to_png_bytes(&source_img)?;
        log::info!("Photon-rs watermark application completed successfully (in-memory)");

        Ok(result_bytes)
    }

    /// Applies a watermark to an image using photon-rs (file-based processing)
    pub async fn apply_watermark(
        &self,
        source_image_path: &Path,
        logo_image_path: &Path,
        output_path: &Path,
        config: &WatermarkConfig,
    ) -> PhotonResult<()> {
        log::info!("Starting photon-rs watermark application");
        log::info!("Source: {:?}, Logo: {:?}, Output: {:?}", source_image_path, logo_image_path, output_path);
        log::info!("Config: {:?}", config);

        // Validate input paths
        self.validate_path_security(source_image_path)?;
        self.validate_path_security(logo_image_path)?;
        self.validate_path_security(output_path)?;

        // Validate configuration
        self.validate_watermark_config(config)?;

        // Open source image
        log::info!("Opening source image with photon-rs");
        let source_path_str = source_image_path.to_str()
            .ok_or_else(|| PhotonError::InvalidPath("Source image path contains invalid UTF-8".to_string()))?;
        let mut source_img = open_image(source_path_str)
            .map_err(|e| PhotonError::ImageOpen(format!("Failed to load source image '{}': {}", source_path_str, e)))?;

        // Open logo image
        log::info!("Opening logo image with photon-rs");
        let logo_path_str = logo_image_path.to_str()
            .ok_or_else(|| PhotonError::InvalidPath("Logo image path contains invalid UTF-8".to_string()))?;
        let mut logo_img = open_image(logo_path_str)
            .map_err(|e| PhotonError::ImageOpen(format!("Failed to load logo image '{}': {}", logo_path_str, e)))?;

        // Get image dimensions
        let source_width = source_img.get_width();
        let source_height = source_img.get_height();
        let logo_width = logo_img.get_width();
        let logo_height = logo_img.get_height();
        
        log::info!("Source image dimensions: {}x{}", source_width, source_height);
        log::info!("Logo image dimensions: {}x{}", logo_width, logo_height);

        // Resize logo according to configuration
        logo_img = self.resize_logo(&logo_img, &config.size, source_width, source_height)?;
        let resized_logo_width = logo_img.get_width();
        let resized_logo_height = logo_img.get_height();
        log::info!("Resized logo dimensions: {}x{}", resized_logo_width, resized_logo_height);

        // Apply opacity to logo
        if config.opacity < 1.0 {
            log::info!("Applying opacity {} to logo", config.opacity);
            self.apply_opacity(&mut logo_img, config.opacity as f64)?;
        }

        // Calculate position for logo placement
        let (x, y) = self.calculate_position(
            &config.position,
            source_width,
            source_height,
            resized_logo_width,
            resized_logo_height,
        )?;

        log::info!("Positioning logo at ({}, {})", x, y);

        // Composite logo onto source image
        self.composite_images(&mut source_img, &logo_img, x, y)?;

        // Save the result
        log::info!("Saving watermarked image to: {:?}", output_path);
        let output_path_str = output_path.to_str()
            .ok_or_else(|| PhotonError::InvalidPath("Output image path contains invalid UTF-8".to_string()))?;
        save_image(source_img, output_path_str)
            .map_err(|e| PhotonError::ImageSave(format!("Failed to save watermarked image to '{}': {}", output_path_str, e)))?;

        log::info!("Photon-rs watermark application completed successfully");
        Ok(())
    }

    /// Resizes a logo according to the watermark size configuration
    fn resize_logo(
        &self,
        logo: &PhotonImage,
        size_config: &WatermarkSize,
        source_width: u32,
        source_height: u32,
    ) -> PhotonResult<PhotonImage> {
        let (target_width, target_height) = match size_config {
            WatermarkSize::Percentage(percent) => {
                // Calculate size as percentage of source image width (maintaining aspect ratio)
                let scale_factor = (*percent as f64) / 100.0;
                let target_width = (source_width as f64 * scale_factor) as u32;
                let aspect_ratio = logo.get_height() as f64 / logo.get_width() as f64;
                let target_height = (target_width as f64 * aspect_ratio) as u32;
                (target_width, target_height)
            }
            WatermarkSize::Absolute { width, height } => (*width, *height),
            WatermarkSize::FitWidth(width) => {
                let aspect_ratio = logo.get_height() as f64 / logo.get_width() as f64;
                let target_height = (*width as f64 * aspect_ratio) as u32;
                (*width, target_height)
            }
            WatermarkSize::FitHeight(height) => {
                let aspect_ratio = logo.get_width() as f64 / logo.get_height() as f64;
                let target_width = (*height as f64 * aspect_ratio) as u32;
                (target_width, *height)
            }
        };

        log::info!("Resizing logo from {}x{} to {}x{}", 
                  logo.get_width(), logo.get_height(), target_width, target_height);

        // Use photon-rs resize function
        let resized = photon_rs::transform::resize(logo, target_width, target_height, photon_rs::transform::SamplingFilter::Nearest);
        Ok(resized)
    }

    /// Applies opacity to an image by modifying the alpha channel
    fn apply_opacity(&self, image: &mut PhotonImage, opacity: f64) -> PhotonResult<()> {
        let opacity_u8 = (opacity * 255.0) as u8;
        
        // Get raw pixel data
        let raw_pixels = image.get_raw_pixels();
        let width = image.get_width() as usize;
        let height = image.get_height() as usize;
        
        // Create new pixel data with modified alpha
        let mut new_pixels = Vec::with_capacity(raw_pixels.len());
        
        for chunk in raw_pixels.chunks(4) {
            if chunk.len() == 4 {
                let r = chunk[0];
                let g = chunk[1];
                let b = chunk[2];
                let original_alpha = chunk[3];
                
                // Apply opacity to alpha channel
                let new_alpha = ((original_alpha as f64 * opacity) as u8).min(opacity_u8);
                
                new_pixels.extend_from_slice(&[r, g, b, new_alpha]);
            }
        }
        
        // Create new PhotonImage with modified pixels
        *image = PhotonImage::new(new_pixels, width as u32, height as u32);
        
        Ok(())
    }

    /// Calculates the position for logo placement based on configuration
    fn calculate_position(
        &self,
        position: &WatermarkPosition,
        source_width: u32,
        source_height: u32,
        logo_width: u32,
        logo_height: u32,
    ) -> PhotonResult<(u32, u32)> {
        let (x, y) = match position {
            WatermarkPosition::Corner(corner) => {
                let offset = 10u32; // 10px offset from edges
                match corner {
                    CornerPosition::TopLeft => (offset, offset),
                    CornerPosition::TopRight => (source_width.saturating_sub(logo_width + offset), offset),
                    CornerPosition::BottomLeft => (offset, source_height.saturating_sub(logo_height + offset)),
                    CornerPosition::BottomRight => (
                        source_width.saturating_sub(logo_width + offset),
                        source_height.saturating_sub(logo_height + offset)
                    ),
                }
            }
            WatermarkPosition::Edge(edge) => {
                let offset = 10u32; // 10px offset from edges
                match edge {
                    EdgePosition::Top => (
                        (source_width.saturating_sub(logo_width)) / 2,
                        offset
                    ),
                    EdgePosition::Bottom => (
                        (source_width.saturating_sub(logo_width)) / 2,
                        source_height.saturating_sub(logo_height + offset)
                    ),
                    EdgePosition::Left => (
                        offset,
                        (source_height.saturating_sub(logo_height)) / 2
                    ),
                    EdgePosition::Right => (
                        source_width.saturating_sub(logo_width + offset),
                        (source_height.saturating_sub(logo_height)) / 2
                    ),
                }
            }
            WatermarkPosition::Center => (
                (source_width.saturating_sub(logo_width)) / 2,
                (source_height.saturating_sub(logo_height)) / 2,
            ),
            WatermarkPosition::Custom { x_percent, y_percent } => {
                // Calculate position as percentage of source dimensions
                // Use the percentage coordinate as the top-left corner of the logo (no centering)
                let x = (source_width as f64 * (*x_percent as f64) / 100.0) as u32;
                let y = (source_height as f64 * (*y_percent as f64) / 100.0) as u32;
                (x, y)
            }
        };

        // Ensure position is within bounds
        let x = x.min(source_width.saturating_sub(logo_width));
        let y = y.min(source_height.saturating_sub(logo_height));

        Ok((x, y))
    }

    /// Composites logo image onto source image at specified position
    fn composite_images(
        &self,
        source: &mut PhotonImage,
        logo: &PhotonImage,
        x: u32,
        y: u32,
    ) -> PhotonResult<()> {
        log::info!("Compositing logo onto source image at position ({}, {})", x, y);

        // Get dimensions
        let source_width = source.get_width();
        let source_height = source.get_height();
        let logo_width = logo.get_width();
        let logo_height = logo.get_height();

        // Validate position bounds
        if x + logo_width > source_width || y + logo_height > source_height {
            return Err(PhotonError::ProcessingFailed(
                format!("Logo position ({}, {}) with size {}x{} exceeds source image bounds {}x{}",
                       x, y, logo_width, logo_height, source_width, source_height)
            ));
        }

        // Get raw pixel data
        let source_pixels = source.get_raw_pixels();
        let logo_pixels = logo.get_raw_pixels();

        // Create new pixel buffer
        let mut new_pixels = source_pixels.clone();

        // Composite logo pixels onto source
        for logo_y in 0..logo_height {
            for logo_x in 0..logo_width {
                let source_pixel_x = x + logo_x;
                let source_pixel_y = y + logo_y;

                // Calculate pixel indices
                let logo_idx = ((logo_y * logo_width + logo_x) * 4) as usize;
                let source_idx = ((source_pixel_y * source_width + source_pixel_x) * 4) as usize;

                if logo_idx + 3 < logo_pixels.len() && source_idx + 3 < new_pixels.len() {
                    let logo_r = logo_pixels[logo_idx] as f64;
                    let logo_g = logo_pixels[logo_idx + 1] as f64;
                    let logo_b = logo_pixels[logo_idx + 2] as f64;
                    let logo_a = logo_pixels[logo_idx + 3] as f64 / 255.0;

                    let source_r = new_pixels[source_idx] as f64;
                    let source_g = new_pixels[source_idx + 1] as f64;
                    let source_b = new_pixels[source_idx + 2] as f64;
                    let source_a = new_pixels[source_idx + 3] as f64 / 255.0;

                    // Alpha blending
                    let out_a = logo_a + source_a * (1.0 - logo_a);
                    let out_r = if out_a > 0.0 {
                        (logo_r * logo_a + source_r * source_a * (1.0 - logo_a)) / out_a
                    } else {
                        0.0
                    };
                    let out_g = if out_a > 0.0 {
                        (logo_g * logo_a + source_g * source_a * (1.0 - logo_a)) / out_a
                    } else {
                        0.0
                    };
                    let out_b = if out_a > 0.0 {
                        (logo_b * logo_a + source_b * source_a * (1.0 - logo_a)) / out_a
                    } else {
                        0.0
                    };

                    // Set blended pixel
                    new_pixels[source_idx] = out_r.min(255.0) as u8;
                    new_pixels[source_idx + 1] = out_g.min(255.0) as u8;
                    new_pixels[source_idx + 2] = out_b.min(255.0) as u8;
                    new_pixels[source_idx + 3] = (out_a * 255.0).min(255.0) as u8;
                }
            }
        }

        // Update source image with composited pixels
        *source = PhotonImage::new(new_pixels, source_width, source_height);

        Ok(())
    }

    /// Validates path security to prevent directory traversal attacks
    fn validate_path_security(&self, path: &Path) -> PhotonResult<()> {
        let path_str = path.to_string_lossy();
        
        // Check for directory traversal patterns
        if path_str.contains("..") {
            return Err(PhotonError::InvalidPath(
                format!("Path contains directory traversal: {}", path_str)
            ));
        }
        
        // Check for absolute paths outside temp directories
        if path.is_absolute() {
            let temp_dir = std::env::temp_dir();
            if !path.starts_with(&temp_dir) {
                return Err(PhotonError::InvalidPath(
                    format!("Path outside allowed temp directory: {}", path_str)
                ));
            }
        }
        
        // Check for suspicious characters
        if path_str.contains(';') || path_str.contains('|') || path_str.contains('&') {
            return Err(PhotonError::InvalidPath(
                format!("Path contains suspicious characters: {}", path_str)
            ));
        }
        
        Ok(())
    }

    /// Converts a PhotonImage to PNG bytes by saving to a temporary file and reading it back
    fn photon_image_to_png_bytes(&self, photon_img: &PhotonImage) -> PhotonResult<Vec<u8>> {
        // Create a temporary file path
        let temp_id = uuid::Uuid::new_v4();
        let temp_path = std::env::temp_dir().join(format!("photon_temp_{}.png", temp_id));
        
        // Save the PhotonImage to a temporary PNG file
        save_image(photon_img.clone(), &temp_path)
            .map_err(|e| PhotonError::ImageProcessing(format!("Failed to save temporary PNG: {}", e)))?;
        
        // Read the PNG file back as bytes
        let png_bytes = std::fs::read(&temp_path)
            .map_err(|e| PhotonError::Io(e))?;
        
        // Clean up the temporary file
        if let Err(e) = std::fs::remove_file(&temp_path) {
            log::warn!("Failed to clean up temporary PNG file {:?}: {}", temp_path, e);
        }
        
        Ok(png_bytes)
    }

    /// Validates watermark configuration for security and sanity
    fn validate_watermark_config(&self, config: &WatermarkConfig) -> PhotonResult<()> {
        // Validate opacity range
        if config.opacity < 0.0 || config.opacity > 1.0 {
            return Err(PhotonError::InvalidConfig(
                format!("Opacity must be between 0.0 and 1.0, got: {}", config.opacity)
            ));
        }
        
        // Validate size configuration
        match &config.size {
            WatermarkSize::Percentage(percent) => {
                if *percent <= 0.0 || *percent > 200.0 {
                    return Err(PhotonError::InvalidConfig(
                        format!("Percentage must be between 0.1 and 200.0, got: {}", percent)
                    ));
                }
            }
            WatermarkSize::Absolute { width, height } => {
                if *width == 0 || *height == 0 || *width > 10000 || *height > 10000 {
                    return Err(PhotonError::InvalidConfig(
                        format!("Absolute dimensions must be between 1 and 10000 pixels, got: {}x{}", width, height)
                    ));
                }
            }
            WatermarkSize::FitWidth(width) => {
                if *width == 0 || *width > 10000 {
                    return Err(PhotonError::InvalidConfig(
                        format!("Width must be between 1 and 10000 pixels, got: {}", width)
                    ));
                }
            }
            WatermarkSize::FitHeight(height) => {
                if *height == 0 || *height > 10000 {
                    return Err(PhotonError::InvalidConfig(
                        format!("Height must be between 1 and 10000 pixels, got: {}", height)
                    ));
                }
            }
        }
        
        // Validate position configuration
        if let WatermarkPosition::Custom { x_percent, y_percent } = &config.position {
            if *x_percent < 0.0 || *x_percent > 100.0 || *y_percent < 0.0 || *y_percent > 100.0 {
                return Err(PhotonError::InvalidConfig(
                    format!("Custom position percentages must be between 0.0 and 100.0, got: {}%, {}%", x_percent, y_percent)
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::watermark_schemas::*;

    #[test]
    fn test_path_security_validation() {
        let processor = PhotonProcessor::new();
        let temp_dir = std::env::temp_dir();
        let safe_path = temp_dir.join("safe_file.jpg");
        let traversal_path = temp_dir.join("../../../etc/passwd");
        let suspicious_path = temp_dir.join("file;rm -rf /");

        // Safe path should pass
        assert!(processor.validate_path_security(&safe_path).is_ok());
        
        // Directory traversal should fail
        assert!(processor.validate_path_security(&traversal_path).is_err());
        
        // Suspicious characters should fail
        assert!(processor.validate_path_security(&suspicious_path).is_err());
    }

    #[test]
    fn test_watermark_config_validation() {
        let processor = PhotonProcessor::new();

        // Valid config should pass
        let valid_config = WatermarkConfig {
            position: WatermarkPosition::Corner(CornerPosition::BottomRight),
            size: WatermarkSize::Percentage(15.0),
            opacity: 0.8,
        };
        assert!(processor.validate_watermark_config(&valid_config).is_ok());

        // Invalid opacity should fail
        let invalid_opacity = WatermarkConfig {
            position: WatermarkPosition::Corner(CornerPosition::BottomRight),
            size: WatermarkSize::Percentage(15.0),
            opacity: 1.5,
        };
        assert!(processor.validate_watermark_config(&invalid_opacity).is_err());

        // Invalid percentage should fail
        let invalid_percentage = WatermarkConfig {
            position: WatermarkPosition::Corner(CornerPosition::BottomRight),
            size: WatermarkSize::Percentage(250.0),
            opacity: 0.8,
        };
        assert!(processor.validate_watermark_config(&invalid_percentage).is_err());

        // Invalid custom position should fail
        let invalid_position = WatermarkConfig {
            position: WatermarkPosition::Custom { x_percent: 150.0, y_percent: 50.0 },
            size: WatermarkSize::Percentage(15.0),
            opacity: 0.8,
        };
        assert!(processor.validate_watermark_config(&invalid_position).is_err());
    }

    #[test]
    fn test_position_calculation() {
        let processor = PhotonProcessor::new();
        let source_width = 1000u32;
        let source_height = 600u32;
        let logo_width = 100u32;
        let logo_height = 60u32;

        // Test corner positions
        let top_left = WatermarkPosition::Corner(CornerPosition::TopLeft);
        let (x, y) = processor.calculate_position(&top_left, source_width, source_height, logo_width, logo_height).unwrap();
        assert_eq!((x, y), (10, 10));

        let bottom_right = WatermarkPosition::Corner(CornerPosition::BottomRight);
        let (x, y) = processor.calculate_position(&bottom_right, source_width, source_height, logo_width, logo_height).unwrap();
        assert_eq!((x, y), (890, 530));

        // Test center position
        let center = WatermarkPosition::Center;
        let (x, y) = processor.calculate_position(&center, source_width, source_height, logo_width, logo_height).unwrap();
        assert_eq!((x, y), (450, 270));

        // Test custom position (top-left corner of logo placed at percentage coordinate)
        let custom = WatermarkPosition::Custom { x_percent: 50.0, y_percent: 50.0 };
        let (x, y) = processor.calculate_position(&custom, source_width, source_height, logo_width, logo_height).unwrap();
        assert_eq!((x, y), (500, 300));
    }
}
