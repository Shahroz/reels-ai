//! Utility for extracting GCS URIs from any text content.
//!
//! This module provides functions to extract Google Cloud Storage URIs from any string
//! or JSON content using regex patterns. It can handle any format that contains GCS URLs.

use serde_json::Value;
use regex::Regex;

/// Extracts GCS URIs from any text content using regex patterns
/// 
/// This function can extract GCS URLs from any string format - whether it's JSON, 
/// plain text, logs, or any other format. It uses regex patterns to find all GCS URLs
/// and automatically deduplicates them.
/// 
/// Supports both formats:
/// - `gs://bucket-name/path/to/file.ext`
/// - `https://storage.googleapis.com/bucket-name/path/to/file.ext`
/// 
/// # Arguments
/// 
/// * `response` - Any JSON value that might contain GCS URLs
/// 
/// # Returns
/// 
/// A `Result` containing a deduplicated vector of GCS URI strings, or an error message if none found.
pub fn extract_gcs_uris_from_response(response: &Value) -> Result<Vec<String>, String> {
    // Convert the entire JSON response to a string to search through
    let response_text = serde_json::to_string(response)
        .map_err(|e| format!("Failed to serialize response to string: {e}"))?;
    
    extract_gcs_uris_from_text(&response_text)
}

/// Extracts GCS URIs from any text string using regex patterns
/// 
/// This is the core function that can find GCS URLs in any text format.
/// It uses comprehensive regex patterns to match both gs:// and https://storage.googleapis.com/ URLs.
/// 
/// # Arguments
/// 
/// * `text` - Any string that might contain GCS URLs
/// 
/// # Returns
/// 
/// A `Result` containing a deduplicated vector of GCS URI strings, or an error message if none found.
pub fn extract_gcs_uris_from_text(text: &str) -> Result<Vec<String>, String> {
    let mut gcs_uris = std::collections::HashSet::new();
    
    // Regex pattern for gs:// URLs
    // Matches: gs://bucket-name/path/to/file.ext
    // Stops at whitespace, quotes, or common JSON delimiters
    let gs_pattern = Regex::new(r#"gs://[a-zA-Z0-9.\-_]+(?:/[^"'\s\],}]*)*"#)
        .map_err(|e| format!("Failed to compile gs:// regex: {e}"))?;
    
    // Regex pattern for https://storage.googleapis.com/ URLs  
    // Matches: https://storage.googleapis.com/bucket-name/path/to/file.ext
    // Stops at whitespace, quotes, or common JSON delimiters
    let https_pattern = Regex::new(r#"https://storage\.googleapis\.com/[a-zA-Z0-9.\-_]+(?:/[^"'\s\],}]*)*"#)
        .map_err(|e| format!("Failed to compile https:// regex: {e}"))?;
    
    // Find all gs:// URLs
    for captures in gs_pattern.find_iter(text) {
        let url = captures.as_str();
        if is_valid_gcs_url(url) {
            gcs_uris.insert(url.to_string());
        }
    }
    
    // Find all https://storage.googleapis.com/ URLs
    for captures in https_pattern.find_iter(text) {
        let url = captures.as_str();
        if is_valid_gcs_url(url) {
            gcs_uris.insert(url.to_string());
        }
    }
    
    if gcs_uris.is_empty() {
        log::debug!("No GCS URIs found in text. Text sample (first 200 chars): {}", 
                   &text.chars().take(200).collect::<String>());
        return Err("No GCS URIs found in text content".to_string());
    }
    
    // Convert HashSet to Vec and sort for consistent ordering
    let mut uris: Vec<String> = gcs_uris.into_iter().collect();
    uris.sort();
    
    log::debug!("Extracted {} unique GCS URI(s) from text: {:?}", uris.len(), uris);
    Ok(uris)
}

/// Helper function to validate if a string is a properly formatted GCS URL
fn is_valid_gcs_url(url: &str) -> bool {
    if let Some(without_prefix) = url.strip_prefix("gs://") {
        // Validate gs:// format: gs://bucket/path
        // Remove "gs://"
        let parts: Vec<&str> = without_prefix.splitn(2, '/').collect();
        !parts[0].is_empty() && !parts[0].contains(' ')
    } else if url.starts_with("https://storage.googleapis.com/") {
        // Validate https:// format: https://storage.googleapis.com/bucket/path
        let without_prefix = &url[32..]; // Remove "https://storage.googleapis.com/"
        let parts: Vec<&str> = without_prefix.splitn(2, '/').collect();
        !parts[0].is_empty() && !parts[0].contains(' ')
    } else {
        false
    }
}



/// Extracts the GCS object name from a GCS URL
/// 
/// Handles both gs:// and https://storage.googleapis.com/ URL formats.
/// 
/// # Arguments
/// 
/// * `gcs_url` - The full GCS URL
/// 
/// # Returns
/// 
/// A `Result` containing the object name (path), or an error message.
/// 
/// # Examples
/// 
/// ```ignore
/// let object_name = extract_object_name_from_gcs_url("gs://my-bucket/path/to/file.jpg")?;
/// assert_eq!(object_name, "path/to/file.jpg");
/// 
/// let object_name = extract_object_name_from_gcs_url("https://storage.googleapis.com/my-bucket/path/to/file.jpg")?;
/// assert_eq!(object_name, "path/to/file.jpg");
/// ```
pub fn extract_object_name_from_gcs_url(gcs_url: &str) -> Result<String, String> {
    if let Some(path) = gcs_url.strip_prefix("gs://") {
        // Format: gs://bucket/object/path
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() == 2 && !parts[1].is_empty() {
            Ok(parts[1].to_string())
        } else {
            Err(format!("Invalid gs:// URL format: {gcs_url}"))
        }
    } else if let Some(path) = gcs_url.strip_prefix("https://storage.googleapis.com/") {
        // Format: https://storage.googleapis.com/bucket/object/path
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() == 2 && !parts[1].is_empty() {
            Ok(parts[1].to_string())
        } else {
            Err(format!("Invalid https://storage.googleapis.com/ URL format: {gcs_url}"))
        }
    } else {
        Err(format!("Unsupported GCS URL format: {gcs_url}. Must start with 'gs://' or 'https://storage.googleapis.com/'"))
    }
}

/// Extracts the bucket name from a GCS URL
/// 
/// # Arguments
/// 
/// * `gcs_url` - The full GCS URL
/// 
/// # Returns
/// 
/// A `Result` containing the bucket name, or an error message.
pub fn extract_bucket_name_from_gcs_url(gcs_url: &str) -> Result<String, String> {
    if let Some(path) = gcs_url.strip_prefix("gs://") {
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if !parts[0].is_empty() {
            Ok(parts[0].to_string())
        } else {
            Err(format!("Invalid gs:// URL format: {gcs_url}"))
        }
    } else if let Some(path) = gcs_url.strip_prefix("https://storage.googleapis.com/") {
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if !parts[0].is_empty() {
            Ok(parts[0].to_string())
        } else {
            Err(format!("Invalid https://storage.googleapis.com/ URL format: {gcs_url}"))
        }
    } else {
        Err(format!("Unsupported GCS URL format: {gcs_url}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_gcs_uris_from_simple_response() {
        let response = json!({
            "result": "gs://my-bucket/enhanced/image1.jpg"
        });
        
        let uris = extract_gcs_uris_from_response(&response).unwrap();
        assert_eq!(uris.len(), 1);
        assert_eq!(uris[0], "gs://my-bucket/enhanced/image1.jpg");
    }

    #[test]
    fn test_extract_gcs_uris_from_nested_response() {
        let response = json!({
            "workflow": {
                "results": [
                    {
                        "type": "image",
                        "url": "https://storage.googleapis.com/my-bucket/enhanced/image1.jpg"
                    },
                    {
                        "type": "image", 
                        "url": "gs://my-bucket/enhanced/image2.jpg"
                    }
                ]
            }
        });
        
        let uris = extract_gcs_uris_from_response(&response).unwrap();
        assert_eq!(uris.len(), 2);
        assert!(uris.contains(&"https://storage.googleapis.com/my-bucket/enhanced/image1.jpg".to_string()));
        assert!(uris.contains(&"gs://my-bucket/enhanced/image2.jpg".to_string()));
    }

    #[test]
    fn test_extract_object_name_from_gs_url() {
        let object_name = extract_object_name_from_gcs_url("gs://my-bucket/path/to/file.jpg").unwrap();
        assert_eq!(object_name, "path/to/file.jpg");
    }

    #[test]
    fn test_extract_object_name_from_https_url() {
        let object_name = extract_object_name_from_gcs_url("https://storage.googleapis.com/my-bucket/path/to/file.jpg").unwrap();
        assert_eq!(object_name, "path/to/file.jpg");
    }

    #[test]
    fn test_extract_bucket_name_from_gs_url() {
        let bucket_name = extract_bucket_name_from_gcs_url("gs://my-bucket/path/to/file.jpg").unwrap();
        assert_eq!(bucket_name, "my-bucket");
    }

    #[test]
    fn test_extract_bucket_name_from_https_url() {
        let bucket_name = extract_bucket_name_from_gcs_url("https://storage.googleapis.com/my-bucket/path/to/file.jpg").unwrap();
        assert_eq!(bucket_name, "my-bucket");
    }

    #[test]
    fn test_invalid_url_formats() {
        assert!(extract_object_name_from_gcs_url("http://example.com/file.jpg").is_err());
        assert!(extract_object_name_from_gcs_url("gs://bucket-only").is_err());
        assert!(extract_bucket_name_from_gcs_url("invalid-url").is_err());
    }

    #[test]
    fn test_extract_from_any_json_format() {
        // Test with the actual response format from Gennodes workflow
        let response = json!({
            "result": [
                {
                    "id": "db67ee09-d41c-4d9a-985f-36e3cdb89e01",
                    "data": {
                        "AnyJSON": {
                            "value": ["https://storage.googleapis.com/real-estate-videos/IMG_6573_1_37_frame_26_20250807_152854_046_retouch.png"]
                        }
                    },
                    "sources": [
                        {
                            "Url": "https://storage.googleapis.com/real-estate-videos/IMG_6573_1_37_frame_26_20250807_152854_046_retouch.png"
                        }
                    ]
                }
            ]
        });

        let uris = extract_gcs_uris_from_response(&response).unwrap();
        
        // Should find the URL and deduplicate (it appears in both data.AnyJSON.value and sources)
        assert_eq!(uris.len(), 1);
        assert_eq!(uris[0], "https://storage.googleapis.com/real-estate-videos/IMG_6573_1_37_frame_26_20250807_152854_046_retouch.png");
    }

    #[test]
    fn test_extract_from_plain_text() {
        let text = "Here are your images: gs://my-bucket/image1.png and https://storage.googleapis.com/another-bucket/image2.jpg";
        
        let uris = extract_gcs_uris_from_text(text).unwrap();
        
        assert_eq!(uris.len(), 2);
        assert!(uris.contains(&"gs://my-bucket/image1.png".to_string()));
        assert!(uris.contains(&"https://storage.googleapis.com/another-bucket/image2.jpg".to_string()));
    }

    #[test] 
    fn test_extract_from_log_format() {
        let log_text = r#"
        [2025-01-08] Processing complete.
        Output saved to: gs://real-estate/processed/image_enhanced.webp
        Backup available at https://storage.googleapis.com/backup-bucket/image_backup.webp
        Status: SUCCESS
        "#;
        
        let uris = extract_gcs_uris_from_text(log_text).unwrap();
        
        assert_eq!(uris.len(), 2);
        assert!(uris.contains(&"gs://real-estate/processed/image_enhanced.webp".to_string()));
        assert!(uris.contains(&"https://storage.googleapis.com/backup-bucket/image_backup.webp".to_string()));
    }

    #[test]
    fn test_extract_with_special_characters() {
        let text = r#"Files: "gs://bucket-name/path_without_spaces/file.png", 'https://storage.googleapis.com/my-bucket/file_with_underscores.jpg'"#;
        
        let uris = extract_gcs_uris_from_text(text).unwrap();
        
        assert_eq!(uris.len(), 2);
        assert!(uris.contains(&"gs://bucket-name/path_without_spaces/file.png".to_string()));
        assert!(uris.contains(&"https://storage.googleapis.com/my-bucket/file_with_underscores.jpg".to_string()));
    }

    #[test]
    fn test_complex_json_with_multiple_urls() {
        // Test with multiple results that might have different URLs
        let response = json!({
            "result": [
                {
                    "id": "result-1",
                    "data": {
                        "AnyJSON": {
                            "value": [
                                "https://storage.googleapis.com/bucket/image1.png",
                                "https://storage.googleapis.com/bucket/image2.png"
                            ]
                        }
                    },
                    "sources": [
                        {"Url": "https://storage.googleapis.com/bucket/image1.png"}
                    ]
                },
                {
                    "id": "result-2", 
                    "data": {
                        "AnyJSON": {
                            "value": ["gs://bucket/image3.png"]
                        }
                    },
                    "sources": [
                        {"Url": "gs://bucket/image3.png"},
                        {"Url": "https://storage.googleapis.com/bucket/image4.png"}
                    ]
                }
            ]
        });

        let uris = extract_gcs_uris_from_response(&response).unwrap();
        
        // Should find all unique URLs and sort them
        assert_eq!(uris.len(), 4);
        assert!(uris.contains(&"gs://bucket/image3.png".to_string()));
        assert!(uris.contains(&"https://storage.googleapis.com/bucket/image1.png".to_string()));
        assert!(uris.contains(&"https://storage.googleapis.com/bucket/image2.png".to_string()));
        assert!(uris.contains(&"https://storage.googleapis.com/bucket/image4.png".to_string()));
        
        // Should be sorted
        let mut expected = uris.clone();
        expected.sort();
        assert_eq!(uris, expected);
    }

    #[test]
    fn test_any_json_structure() {
        // Test with completely unknown/custom JSON structure  
        let response = json!({
            "custom_format": {
                "nested": {
                    "message": "Files saved to gs://test-bucket/output.jpg and backup at https://storage.googleapis.com/backup/file.png"
                }
            },
            "other_data": "Another URL here: gs://another-bucket/file2.webp"
        });

        let uris = extract_gcs_uris_from_response(&response).unwrap();
        
        // Should find URLs anywhere in the JSON when converted to string
        assert_eq!(uris.len(), 3);
        assert!(uris.contains(&"gs://test-bucket/output.jpg".to_string()));
        assert!(uris.contains(&"https://storage.googleapis.com/backup/file.png".to_string()));
        assert!(uris.contains(&"gs://another-bucket/file2.webp".to_string()));
    }

    #[test]
    fn test_deduplication() {
        let text = r#"
        Files: gs://bucket/file.jpg, gs://bucket/file.jpg, gs://bucket/file.jpg
        Also: https://storage.googleapis.com/bucket/file.jpg
        "#;
        
        let uris = extract_gcs_uris_from_text(text).unwrap();
        
        // Should deduplicate identical URLs
        assert_eq!(uris.len(), 2);
        assert!(uris.contains(&"gs://bucket/file.jpg".to_string()));
        assert!(uris.contains(&"https://storage.googleapis.com/bucket/file.jpg".to_string()));
    }

    #[test]
    fn test_edge_cases() {
        let text = r#"
        Valid: gs://valid-bucket/file.png
        Invalid: gs:// (empty bucket)
        Invalid: gs://bucket_with_spaces/file.png (should work with underscores)
        Valid: https://storage.googleapis.com/another-bucket/file.jpg
        Invalid: https://storage.googleapis.com/ (empty bucket)
        "#;
        
        let uris = extract_gcs_uris_from_text(text).unwrap();
        
        // Should find all valid URLs (including with underscores)
        assert_eq!(uris.len(), 3);
        assert!(uris.contains(&"gs://valid-bucket/file.png".to_string()));
        assert!(uris.contains(&"gs://bucket_with_spaces/file.png".to_string()));
        assert!(uris.contains(&"https://storage.googleapis.com/another-bucket/file.jpg".to_string()));
    }

    #[test]
    fn test_no_urls_found() {
        let text = "This text has no GCS URLs at all. Just regular text.";
        
        let result = extract_gcs_uris_from_text(text);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No GCS URIs found in text content");
    }
} 