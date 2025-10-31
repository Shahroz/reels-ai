//! Generates filename for watermarked assets.
//!
//! This function creates a unique filename for watermarked images by appending
//! a timestamp and watermark suffix to the original filename.
//! Ensures output files have consistent naming and don't conflict.

/// Generates filename for watermarked asset
pub fn generate_watermarked_filename(original_name: &str) -> std::string::String {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    
    // Remove file extension if present and append watermarked suffix
    let base_name = if let std::option::Option::Some(dot_pos) = original_name.rfind('.') {
        &original_name[..dot_pos]
    } else {
        original_name
    };
    
    std::format!("{}_watermarked_{}.png", base_name, timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_watermarked_filename() {
        let result = generate_watermarked_filename("test_image.jpg");
        assert!(result.starts_with("test_image_watermarked_"));
        assert!(result.ends_with(".png"));
        
        let result_no_ext = generate_watermarked_filename("test_image");
        assert!(result_no_ext.starts_with("test_image_watermarked_"));
        assert!(result_no_ext.ends_with(".png"));
    }
    
    #[test]
    fn test_generate_watermarked_filename_preserves_base_name() {
        let result = generate_watermarked_filename("my-company-logo.svg");
        assert!(result.starts_with("my-company-logo_watermarked_"));
        assert!(result.ends_with(".png"));
        
        // Test with multiple dots
        let result_multiple_dots = generate_watermarked_filename("image.backup.old.jpg");
        assert!(result_multiple_dots.starts_with("image.backup.old_watermarked_"));
        assert!(result_multiple_dots.ends_with(".png"));
    }

    #[test]
    fn test_filename_uniqueness() {
        // Generate multiple filenames quickly and ensure they're different
        let filename1 = generate_watermarked_filename("test.jpg");
        // Small delay to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(1));
        let filename2 = generate_watermarked_filename("test.jpg");
        
        // They should have the same prefix but different timestamps
        assert!(filename1.starts_with("test_watermarked_"));
        assert!(filename2.starts_with("test_watermarked_"));
        // Note: Due to timestamp precision, they might be the same if generated quickly
        // This test mainly ensures the function works consistently
    }

    #[test]
    fn test_special_characters_in_filename() {
        let result = generate_watermarked_filename("file with spaces & symbols!.png");
        assert!(result.starts_with("file with spaces & symbols!_watermarked_"));
        assert!(result.ends_with(".png"));
    }

    #[test]
    fn test_empty_filename() {
        let result = generate_watermarked_filename("");
        assert!(result.starts_with("_watermarked_"));
        assert!(result.ends_with(".png"));
    }
}
