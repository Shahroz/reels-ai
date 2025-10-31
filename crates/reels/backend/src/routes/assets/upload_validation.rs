//! Asset upload validation service.
//!
//! Provides comprehensive validation for asset uploads including file type checks,
//! size limits, name validation, and content type verification.
//! Ensures uploaded assets meet platform requirements and security constraints.

/// Maximum file size allowed for uploads (2GB for videos, smaller for other types)
const MAX_VIDEO_SIZE: u64 = 2 * 1024 * 1024 * 1024; // 2GB
const MAX_IMAGE_SIZE: u64 = 50 * 1024 * 1024; // 50MB
const MAX_DOCUMENT_SIZE: u64 = 100 * 1024 * 1024; // 100MB
const MAX_OTHER_SIZE: u64 = 10 * 1024 * 1024; // 10MB

/// Supported video file extensions
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "avi", "mkv", "webm", "m4v"];

/// Supported image file extensions  
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "heic", "dng"];

/// Supported document file extensions
const DOCUMENT_EXTENSIONS: &[&str] = &["pdf", "doc", "docx", "txt", "md", "rtf"];

/// Result of upload validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub error_message: Option<std::string::String>,
    pub asset_category: AssetCategory,
    pub normalized_extension: std::string::String,
    pub size_limit: u64,
    pub secure_extension: std::string::String, // Derived from content type, not client input
}

/// Categories of assets based on file type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssetCategory {
    Video,
    Image,
    Document,
    Other,
}

/// Validates an asset upload request with security-focused approach
pub fn validate_upload_request(
    file_name: &str,
    file_size: u64,
    content_type: &str,
) -> ValidationResult {
    // 1. Validate file name
    if let Some(error) = validate_file_name(file_name) {
        return ValidationResult {
            is_valid: false,
            error_message: Some(error),
            asset_category: AssetCategory::Other,
            normalized_extension: std::string::String::new(),
            size_limit: MAX_OTHER_SIZE,
            secure_extension: "bin".to_string(),
        };
    }

    // 2. Validate and categorize by content type FIRST (security-focused)
    let category = match determine_asset_category_from_content_type(content_type) {
        AssetCategory::Other => {
            return ValidationResult {
                is_valid: false,
                error_message: Some(format!("Unsupported content type: {content_type}")),
                asset_category: AssetCategory::Other,
                normalized_extension: std::string::String::new(),
                size_limit: MAX_OTHER_SIZE,
                secure_extension: "bin".to_string(),
            };
        }
        cat => cat,
    };

    // 3. Extract client-provided extension for secondary validation
    let client_extension = extract_file_extension(file_name);
    let normalized_client_ext = client_extension.to_lowercase();

    // 4. Verify client extension matches content type category (security check)
    if let Some(error) = validate_extension_matches_content_type(&normalized_client_ext, &category) {
        return ValidationResult {
            is_valid: false,
            error_message: Some(error),
            asset_category: category,
            normalized_extension: normalized_client_ext,
            size_limit: get_size_limit_for_category(&category),
            secure_extension: get_fallback_extension_for_category(&category),
        };
    }

    // 5. Use client extension as secure extension when it's valid for the category
    let secure_extension = normalized_client_ext.clone();

    let size_limit = get_size_limit_for_category(&category);

    // 6. Validate file size
    if file_size > size_limit {
        return ValidationResult {
            is_valid: false,
            error_message: Some(format!(
                "File size {} exceeds maximum allowed size {} for {} files",
                format_file_size(file_size),
                format_file_size(size_limit),
                format_category(&category)
            )),
            asset_category: category,
            normalized_extension: normalized_client_ext,
            size_limit,
            secure_extension,
        };
    }

    ValidationResult {
        is_valid: true,
        error_message: None,
        asset_category: category,
        normalized_extension: normalized_client_ext.clone(),
        size_limit,
        secure_extension: normalized_client_ext,
    }
}

/// Derives category and secure file extension from content type
/// Returns None for unsupported content types
fn derive_category_and_extension_from_content_type(content_type: &str) -> Option<(AssetCategory, std::string::String)> {
    match content_type {
        // Video types
        "video/mp4" => Some((AssetCategory::Video, "mp4".to_string())),
        "video/quicktime" => Some((AssetCategory::Video, "mov".to_string())),
        "video/x-msvideo" => Some((AssetCategory::Video, "avi".to_string())),
        "video/x-matroska" => Some((AssetCategory::Video, "mkv".to_string())),
        "video/webm" => Some((AssetCategory::Video, "webm".to_string())),
        "video/x-m4v" => Some((AssetCategory::Video, "m4v".to_string())),
        
        // Image types
        "image/jpeg" => Some((AssetCategory::Image, "jpg".to_string())),
        "image/png" => Some((AssetCategory::Image, "png".to_string())),
        "image/gif" => Some((AssetCategory::Image, "gif".to_string())),
        "image/webp" => Some((AssetCategory::Image, "webp".to_string())),
        "image/svg+xml" => Some((AssetCategory::Image, "svg".to_string())),
        "image/bmp" => Some((AssetCategory::Image, "bmp".to_string())),
        "image/heic" => Some((AssetCategory::Image, "heic".to_string())),
        "image/x-adobe-dng" => Some((AssetCategory::Image, "dng".to_string())),
        
        // Document types
        "application/pdf" => Some((AssetCategory::Document, "pdf".to_string())),
        "application/msword" => Some((AssetCategory::Document, "doc".to_string())),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => Some((AssetCategory::Document, "docx".to_string())),
        "text/plain" => Some((AssetCategory::Document, "txt".to_string())),
        "text/markdown" => Some((AssetCategory::Document, "md".to_string())),
        "application/rtf" => Some((AssetCategory::Document, "rtf".to_string())),
        
        // Unsupported types
        _ => None,
    }
}

/// Validates that client-provided extension matches the content type category
fn validate_extension_matches_content_type(client_extension: &str, category: &AssetCategory) -> Option<std::string::String> {
    let expected_extensions = match category {
        AssetCategory::Video => VIDEO_EXTENSIONS,
        AssetCategory::Image => IMAGE_EXTENSIONS,
        AssetCategory::Document => DOCUMENT_EXTENSIONS,
        AssetCategory::Other => return None, // No validation for other types
    };

    if !expected_extensions.contains(&client_extension) {
        return Some(format!(
            "File extension '{}' does not match content type category {}. Expected one of: {}",
            client_extension,
            format_category(category),
            expected_extensions.join(", ")
        ));
    }

    None
}

fn validate_file_name(file_name: &str) -> Option<std::string::String> {
    if file_name.is_empty() {
        return Some("File name cannot be empty".to_string());
    }

    if file_name.len() > 255 {
        return Some("File name cannot exceed 255 characters".to_string());
    }

    // Check for dangerous characters
    let dangerous_chars = ['/', '\\', '?', '*', ':', '|', '"', '<', '>'];
    for char in dangerous_chars {
        if file_name.contains(char) {
            return Some(format!("File name contains invalid character: {char}"));
        }
    }

    // Must have an extension
    if !file_name.contains('.') {
        return Some("File name must include an extension".to_string());
    }

    None
}

fn extract_file_extension(file_name: &str) -> &str {
    file_name.split('.').next_back().unwrap_or("")
}

fn determine_asset_category(extension: &str) -> AssetCategory {
    if VIDEO_EXTENSIONS.contains(&extension) {
        AssetCategory::Video
    } else if IMAGE_EXTENSIONS.contains(&extension) {
        AssetCategory::Image
    } else if DOCUMENT_EXTENSIONS.contains(&extension) {
        AssetCategory::Document
    } else {
        AssetCategory::Other
    }
}

/// Determines asset category from content type
pub fn determine_asset_category_from_content_type(content_type: &str) -> AssetCategory {
    if content_type.starts_with("video/") {
        AssetCategory::Video
    } else if content_type.starts_with("image/") {
        AssetCategory::Image
    } else if content_type.starts_with("application/pdf") 
        || content_type.starts_with("application/msword")
        || content_type.starts_with("application/vnd.openxmlformats-officedocument.wordprocessingml")
        || content_type.starts_with("text/plain")
        || content_type.starts_with("text/markdown") {
        AssetCategory::Document
    } else {
        AssetCategory::Other
    }
}

fn get_size_limit_for_category(category: &AssetCategory) -> u64 {
    match category {
        AssetCategory::Video => MAX_VIDEO_SIZE,
        AssetCategory::Image => MAX_IMAGE_SIZE,
        AssetCategory::Document => MAX_DOCUMENT_SIZE,
        AssetCategory::Other => MAX_OTHER_SIZE,
    }
}

fn get_fallback_extension_for_category(category: &AssetCategory) -> std::string::String {
    match category {
        AssetCategory::Video => "mp4".to_string(),
        AssetCategory::Image => "jpg".to_string(),
        AssetCategory::Document => "pdf".to_string(),
        AssetCategory::Other => "bin".to_string(),
    }
}

fn validate_content_type_match(
    extension: &str,
    content_type: &str,
    category: &AssetCategory,
) -> Option<std::string::String> {
    let expected_prefix = match category {
        AssetCategory::Video => "video/",
        AssetCategory::Image => "image/",
        AssetCategory::Document => match extension {
            "pdf" => return check_exact_content_type(content_type, "application/pdf"),
            "doc" | "docx" => "application/",
            "txt" => return check_exact_content_type(content_type, "text/plain"),
            "md" => return check_exact_content_type(content_type, "text/markdown"),
            _ => "application/",
        },
        AssetCategory::Other => return None, // Allow any content type for other files
    };

    if !content_type.starts_with(expected_prefix) {
        return Some(format!(
            "Content type '{}' does not match expected type for {} files (should start with '{}')",
            content_type, format_category(category), expected_prefix
        ));
    }

    None
}

fn check_exact_content_type(actual: &str, expected: &str) -> Option<std::string::String> {
    if actual != expected {
        Some(format!("Content type '{actual}' should be '{expected}'"))
    } else {
        None
    }
}

fn format_category(category: &AssetCategory) -> &'static str {
    match category {
        AssetCategory::Video => "video",
        AssetCategory::Image => "image", 
        AssetCategory::Document => "document",
        AssetCategory::Other => "other",
    }
}

fn format_file_size(size: u64) -> std::string::String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size_f, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    // Test validation logic with various file types and edge cases
    
    #[test]
    fn test_valid_video_upload() {
        let result = super::validate_upload_request("property.mp4", 1024 * 1024, "video/mp4");
        assert!(result.is_valid);
        assert_eq!(result.asset_category, super::AssetCategory::Video);
        assert_eq!(result.normalized_extension, "mp4");
        assert_eq!(result.secure_extension, "mp4");
    }

    #[test]
    fn test_oversized_video() {
        let result = super::validate_upload_request(
            "large.mp4", 
            3 * 1024 * 1024 * 1024, // 3GB
            "video/mp4"
        );
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("exceeds maximum"));
    }

    #[test]
    fn test_invalid_file_name() {
        let result = super::validate_upload_request("file?.mp4", 1024, "video/mp4");
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("invalid character"));
    }

    #[test]
    fn test_content_type_mismatch() {
        let result = super::validate_upload_request("image.jpg", 1024, "video/mp4");
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("does not match content type category"));
    }

    #[test]
    fn test_valid_image_upload() {
        let result = super::validate_upload_request("photo.jpg", 5 * 1024 * 1024, "image/jpeg");
        assert!(result.is_valid);
        assert_eq!(result.asset_category, super::AssetCategory::Image);
        assert_eq!(result.secure_extension, "jpg");
    }

    #[test]
    fn test_valid_heic_upload() {
        let result = super::validate_upload_request("photo.heic", 5 * 1024 * 1024, "image/heic");
        assert!(result.is_valid);
        assert_eq!(result.asset_category, super::AssetCategory::Image);
        assert_eq!(result.secure_extension, "heic");
    }

    #[test]
    fn test_valid_document_upload() {
        let result = super::validate_upload_request("doc.pdf", 10 * 1024 * 1024, "application/pdf");
        assert!(result.is_valid);
        assert_eq!(result.asset_category, super::AssetCategory::Document);
        assert_eq!(result.secure_extension, "pdf");
    }

    #[test]
    fn test_unsupported_content_type() {
        let result = super::validate_upload_request("test.exe", 1024, "application/x-executable");
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("Unsupported content type"));
        assert_eq!(result.secure_extension, "bin");
    }

    #[test]
    fn test_derive_category_and_extension_from_content_type() {
        // Test video types
        assert_eq!(super::derive_category_and_extension_from_content_type("video/mp4"), Some((super::AssetCategory::Video, "mp4".to_string())));
        assert_eq!(super::derive_category_and_extension_from_content_type("video/quicktime"), Some((super::AssetCategory::Video, "mov".to_string())));
        
        // Test image types
        assert_eq!(super::derive_category_and_extension_from_content_type("image/jpeg"), Some((super::AssetCategory::Image, "jpg".to_string())));
        assert_eq!(super::derive_category_and_extension_from_content_type("image/png"), Some((super::AssetCategory::Image, "png".to_string())));
        
        // Test document types
        assert_eq!(super::derive_category_and_extension_from_content_type("application/pdf"), Some((super::AssetCategory::Document, "pdf".to_string())));
        assert_eq!(super::derive_category_and_extension_from_content_type("text/plain"), Some((super::AssetCategory::Document, "txt".to_string())));
        
        // Test unsupported type
        assert_eq!(super::derive_category_and_extension_from_content_type("application/x-executable"), None);
    }

    #[test]
    fn test_extension_spoofing_prevention() {
        // Test file with misleading extension but correct content type
        let result = super::validate_upload_request("malicious.exe.jpg", 1024, "image/jpeg");
        assert!(result.is_valid);
        assert_eq!(result.secure_extension, "jpg"); // Uses secure extension from content type
        
        // Test file with wrong extension for content type
        let result = super::validate_upload_request("video.txt", 1024, "video/mp4");
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("does not match content type category"));
    }

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(super::format_file_size(1024), "1.0 KB");
        assert_eq!(super::format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(super::format_file_size(2 * 1024 * 1024 * 1024), "2.0 GB");
    }

    #[test]
    fn test_determine_asset_category_from_content_type() {
        assert_eq!(super::determine_asset_category_from_content_type("video/mp4"), super::AssetCategory::Video);
        assert_eq!(super::determine_asset_category_from_content_type("video/quicktime"), super::AssetCategory::Video);
        assert_eq!(super::determine_asset_category_from_content_type("image/jpeg"), super::AssetCategory::Image);
        assert_eq!(super::determine_asset_category_from_content_type("image/png"), super::AssetCategory::Image);
        assert_eq!(super::determine_asset_category_from_content_type("application/pdf"), super::AssetCategory::Document);
        assert_eq!(super::determine_asset_category_from_content_type("text/plain"), super::AssetCategory::Document);
        assert_eq!(super::determine_asset_category_from_content_type("application/zip"), super::AssetCategory::Other);
        assert_eq!(super::determine_asset_category_from_content_type("unknown/type"), super::AssetCategory::Other);
    }
} 