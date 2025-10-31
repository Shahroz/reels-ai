//! Gets content type from filename extension.
//!
//! This function maps file extensions to their corresponding MIME types
//! for proper HTTP content type headers and file type identification.
//! Supports common image formats and defaults to PNG for unknown types.

/// Gets content type from filename extension
pub fn get_content_type_from_filename(filename: &str) -> std::string::String {
    let extension = filename.split('.').last().unwrap_or("").to_lowercase();
    
    match extension.as_str() {
        "png" => "image/png".to_string(),
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "webp" => "image/webp".to_string(),
        "svg" => "image/svg+xml".to_string(),
        _ => "image/png".to_string(), // Default to PNG
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_content_type_from_filename() {
        assert_eq!(get_content_type_from_filename("test.png"), "image/png");
        assert_eq!(get_content_type_from_filename("test.jpg"), "image/jpeg");
        assert_eq!(get_content_type_from_filename("test.jpeg"), "image/jpeg");
        assert_eq!(get_content_type_from_filename("test.webp"), "image/webp");
        assert_eq!(get_content_type_from_filename("test.svg"), "image/svg+xml");
        assert_eq!(get_content_type_from_filename("test.unknown"), "image/png");
        
        // Test case insensitive
        assert_eq!(get_content_type_from_filename("test.PNG"), "image/png");
        assert_eq!(get_content_type_from_filename("test.JPG"), "image/jpeg");
        assert_eq!(get_content_type_from_filename("test.JPEG"), "image/jpeg");
        
        // Test no extension
        assert_eq!(get_content_type_from_filename("test"), "image/png");
    }

    #[test]
    fn test_jpeg_variants() {
        assert_eq!(get_content_type_from_filename("image.jpg"), "image/jpeg");
        assert_eq!(get_content_type_from_filename("image.jpeg"), "image/jpeg");
        assert_eq!(get_content_type_from_filename("image.JPG"), "image/jpeg");
        assert_eq!(get_content_type_from_filename("image.JPEG"), "image/jpeg");
    }

    #[test]
    fn test_complex_filenames() {
        assert_eq!(get_content_type_from_filename("path/to/file.with.dots.png"), "image/png");
        assert_eq!(get_content_type_from_filename("file-with-dashes_and_underscores.jpg"), "image/jpeg");
        assert_eq!(get_content_type_from_filename("file with spaces.webp"), "image/webp");
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(get_content_type_from_filename(""), "image/png");
        assert_eq!(get_content_type_from_filename("."), "image/png");
        assert_eq!(get_content_type_from_filename(".."), "image/png");
        assert_eq!(get_content_type_from_filename("filename."), "image/png");
    }

    #[test]
    fn test_supported_formats() {
        let supported_formats = vec![
            ("image.png", "image/png"),
            ("image.jpg", "image/jpeg"),
            ("image.jpeg", "image/jpeg"),
            ("image.webp", "image/webp"),
            ("image.svg", "image/svg+xml"),
        ];

        for (filename, expected_type) in supported_formats {
            assert_eq!(get_content_type_from_filename(filename), expected_type);
        }
    }
}
