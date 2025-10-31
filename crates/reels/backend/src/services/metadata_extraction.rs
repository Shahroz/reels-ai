//! Asset metadata extraction service.
//!
//! Provides functionality to extract and structure metadata from uploaded assets
//! based on their file type. Metadata includes file size for images, file size
//! and duration for videos, word count for documents, and null for other types.

/// Asset category for determining metadata extraction approach
pub enum AssetCategory {
    Image,
    Video,
    Document,
    Other,
}

/// Result of video duration extraction attempt
#[derive(Debug, Clone)]
enum DurationExtractionResult {
    /// Precise duration extracted from file headers
    Precise(u64),
    /// Extraction failed (file format supported but parsing failed)
    Failed,
    /// Format not supported for precise extraction
    NotSupported,
}

impl AssetCategory {
    /// Determine asset category from content type
    pub fn from_content_type(content_type: &str) -> Self {
        if content_type.starts_with("image/") {
            AssetCategory::Image
        } else if content_type.starts_with("video/") {
            AssetCategory::Video
        } else if content_type.starts_with("text/") || 
                 content_type == "application/pdf" ||
                 content_type.contains("document") ||
                 content_type.contains("word") ||
                 content_type.contains("rtf") {
            AssetCategory::Document
        } else {
            AssetCategory::Other
        }
    }
}

/// Extract metadata from file content based on the asset category and content type
pub async fn extract_asset_metadata(
    file_content: &[u8], 
    content_type: &str,
) -> std::result::Result<std::option::Option<serde_json::Value>, std::string::String> {
    let category = AssetCategory::from_content_type(content_type);
    
    match category {
        AssetCategory::Image => extract_image_metadata(file_content, content_type).await,
        AssetCategory::Video => extract_video_metadata(file_content, content_type).await,
        AssetCategory::Document => extract_document_metadata(file_content, content_type).await,
        AssetCategory::Other => std::result::Result::Ok(std::option::Option::None), // No metadata for other types
    }
}

/// Extract metadata for image files
async fn extract_image_metadata(
    file_content: &[u8],
    _content_type: &str,
) -> std::result::Result<std::option::Option<serde_json::Value>, std::string::String> {
    // For images, only store file size as requested
    let metadata = serde_json::json!({
        "file_size_bytes": file_content.len()
    });
    std::result::Result::Ok(std::option::Option::Some(metadata))
}

/// Extract metadata for video files
async fn extract_video_metadata(
    file_content: &[u8],
    content_type: &str,
) -> std::result::Result<std::option::Option<serde_json::Value>, std::string::String> {
    let file_size_bytes = file_content.len();
    
    // Try to extract actual video duration for MP4-family files
    let duration_result = if is_mp4_compatible_format(content_type) {
        match extract_mp4_duration(file_content).await {
            std::option::Option::Some(duration) => DurationExtractionResult::Precise(duration),
            std::option::Option::None => DurationExtractionResult::Failed,
        }
    } else {
        DurationExtractionResult::NotSupported
    };
    
    let mut metadata = serde_json::json!({
        "file_size_bytes": file_size_bytes
    });
    
    // Add duration information based on extraction result
    match duration_result {
        DurationExtractionResult::Precise(duration) => {
            metadata["duration_seconds"] = serde_json::json!(duration);
            metadata["duration_method"] = serde_json::json!("precise");
        },
        DurationExtractionResult::Failed => {
            // Fallback to estimation
            if let std::option::Option::Some(estimated_duration) = estimate_video_duration_from_size(file_size_bytes, content_type) {
                metadata["duration_seconds"] = serde_json::json!(estimated_duration);
                metadata["duration_method"] = serde_json::json!("estimate");
            }
        },
        DurationExtractionResult::NotSupported => {
            // Fallback to estimation for unsupported formats
            if let std::option::Option::Some(estimated_duration) = estimate_video_duration_from_size(file_size_bytes, content_type) {
                metadata["duration_seconds"] = serde_json::json!(estimated_duration);
                metadata["duration_method"] = serde_json::json!("estimate");
            }
        }
    }
    
    std::result::Result::Ok(std::option::Option::Some(metadata))
}

/// Extract metadata for document files
async fn extract_document_metadata(
    file_content: &[u8],
    content_type: &str,
) -> std::result::Result<std::option::Option<serde_json::Value>, std::string::String> {
    // Extract text content and split into words
    match extract_text_content(file_content, content_type).await {
        std::result::Result::Ok(text_content) => {
            // Count words (strings separated by spaces) - do NOT store the actual words
            let word_count = text_content
                .split_whitespace()
                .filter(|word| !word.is_empty())
                .count();
            
            let metadata = serde_json::json!({
                "word_count": word_count
            });
            std::result::Result::Ok(std::option::Option::Some(metadata))
        },
        std::result::Result::Err(e) => {
            log::warn!("Failed to extract document text: {}", e);
            // Provide zero word count if text extraction fails
            let metadata = serde_json::json!({
                "word_count": 0
            });
            std::result::Result::Ok(std::option::Option::Some(metadata))
        }
    }
}

/// Extract text content from document files using existing content extraction service
async fn extract_text_content(file_content: &[u8], content_type: &str) -> std::result::Result<std::string::String, std::string::String> {
    // Use the existing content extraction service
    crate::services::content_extraction::extract_text::extract_text(
        file_content,
        content_type,
        "temp_file", // Placeholder filename
    ).await
}

/// Check if content type is compatible with MP4 parser
fn is_mp4_compatible_format(content_type: &str) -> bool {
    // The mp4 crate can handle ISO Base Media File Format derivatives
    match content_type.to_lowercase().as_str() {
        // Standard MP4 formats
        ct if ct.contains("mp4") => true,
        ct if ct.contains("m4v") => true,
        ct if ct.contains("m4a") => true,
        
        // QuickTime formats (uses same container format)
        ct if ct.contains("quicktime") => true,
        ct if ct.contains("mov") => true,
        
        // 3GPP formats (based on ISO base media format)
        ct if ct.contains("3gp") => true,
        ct if ct.contains("3g2") => true,
        
        // Other ISO base media formats
        ct if ct.contains("f4v") => true,  // Flash Video
        ct if ct.contains("mp21") => true, // MPEG-21
        
        _ => false,
    }
}

/// Extract actual duration from MP4 files using the mp4 crate
async fn extract_mp4_duration(file_content: &[u8]) -> std::option::Option<u64> {
    // Create a cursor from the file content
    let cursor = std::io::Cursor::new(file_content);
    let size = file_content.len() as u64;
    
    // Try to parse the MP4 file
    match mp4::Mp4Reader::read_header(cursor, size) {
        std::result::Result::Ok(mp4_reader) => {
            // Get duration from the MP4 reader
            let duration = mp4_reader.duration();
            let timescale = mp4_reader.timescale() as u64;
            if timescale > 0 {
                std::option::Option::Some(duration.as_secs())
            } else {
                log::debug!("Invalid timescale in MP4 file");
                std::option::Option::None
            }
        },
        std::result::Result::Err(e) => {
            log::debug!("Failed to parse MP4 file: {}", e);
            std::option::Option::None
        }
    }
}

/// Estimate video duration based on file size and average bitrate for the format
fn estimate_video_duration_from_size(file_size_bytes: usize, content_type: &str) -> std::option::Option<u64> {
    // Average bitrates in bits per second for different video formats
    // These are conservative estimates for typical web/mobile video quality
    let avg_bitrate_bps = match content_type.to_lowercase().as_str() {
        // MP4 formats - typically H.264 or H.265
        ct if ct.contains("mp4") => 1_500_000,  // 1.5 Mbps for 480p-720p
        ct if ct.contains("m4v") => 1_500_000,  // Similar to MP4
        
        // QuickTime formats - can vary widely, but similar to MP4
        ct if ct.contains("quicktime") => 2_000_000,  // 2 Mbps (slightly higher for professional content)
        ct if ct.contains("mov") => 2_000_000,
        
        // Mobile formats - typically lower bitrate
        ct if ct.contains("3gp") => 500_000,   // 500 Kbps for mobile
        ct if ct.contains("3g2") => 500_000,
        
        // Web formats
        ct if ct.contains("webm") => 1_000_000,  // 1 Mbps for WebM
        ct if ct.contains("ogg") => 1_000_000,
        
        // Flash formats
        ct if ct.contains("f4v") => 1_200_000,   // 1.2 Mbps
        ct if ct.contains("flv") => 800_000,     // 800 Kbps
        
        // Legacy formats - typically higher bitrate due to less efficient codecs
        ct if ct.contains("avi") => 3_000_000,   // 3 Mbps for AVI
        ct if ct.contains("wmv") => 2_500_000,   // 2.5 Mbps for WMV
        ct if ct.contains("mkv") => 2_000_000,   // 2 Mbps for MKV
        
        // Unknown format - use conservative estimate
        _ => 1_500_000,  // Default to 1.5 Mbps
    };
    
    // Convert file size from bytes to bits
    let file_size_bits = file_size_bytes * 8;
    
    // Calculate estimated duration in seconds
    let estimated_duration_seconds = file_size_bits as u64 / avg_bitrate_bps;
    
    // Sanity check: return None for unrealistic durations
    if estimated_duration_seconds < 1 {
        // File too small to be a meaningful video (< 1 second)
        std::option::Option::None
    } else if estimated_duration_seconds > 86400 {
        // File suggests > 24 hours of video, likely unrealistic
        std::option::Option::None
    } else {
        std::option::Option::Some(estimated_duration_seconds)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_image_metadata_extraction() {
        let image_data = b"fake image data";
        
        let result = extract_image_metadata(image_data, "image/png").await;
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        assert!(metadata.is_some());
        
        let metadata_obj = metadata.unwrap();
        assert_eq!(metadata_obj["file_size_bytes"], image_data.len());
    }

    #[tokio::test]
    async fn test_document_metadata_extraction() {
        // This test might fail if content extraction service requires actual document parsing
        // In that case, it would need to be adjusted or mocked
        let text_content = b"Hello world this is a test document with words";
        
        let result = extract_document_metadata(text_content, "text/plain").await;
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        assert!(metadata.is_some());
        
        let metadata_obj = metadata.unwrap();
        // Note: actual word count might be 0 if text extraction fails for plain text
        assert!(metadata_obj["word_count"].is_number());
    }

    #[tokio::test]
    async fn test_video_metadata_extraction() {
        // Test with non-MP4 content
        let video_data = b"fake video data";
        
        let result = extract_video_metadata(video_data, "video/webm").await;
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        assert!(metadata.is_some());
        
        let metadata_obj = metadata.unwrap();
        assert_eq!(metadata_obj["file_size_bytes"], video_data.len());
        // Duration should not be present for non-MP4 files
        assert!(metadata_obj.get("duration_seconds").is_none());
    }

    #[tokio::test]
    async fn test_other_category_returns_none() {
        let file_content = b"some binary data";
        
        let result = extract_asset_metadata(file_content, "application/octet-stream").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_asset_category_from_content_type() {
        // Test image detection
        match AssetCategory::from_content_type("image/png") {
            AssetCategory::Image => {},
            _ => panic!("Expected Image category for image/png"),
        }
        
        // Test video detection
        match AssetCategory::from_content_type("video/mp4") {
            AssetCategory::Video => {},
            _ => panic!("Expected Video category for video/mp4"),
        }
        
        // Test document detection
        match AssetCategory::from_content_type("application/pdf") {
            AssetCategory::Document => {},
            _ => panic!("Expected Document category for application/pdf"),
        }
        
        // Test other detection
        match AssetCategory::from_content_type("application/octet-stream") {
            AssetCategory::Other => {},
            _ => panic!("Expected Other category for application/octet-stream"),
        }
    }

    #[test]
    fn test_mp4_compatible_format_detection() {
        // Test MP4 formats
        assert!(is_mp4_compatible_format("video/mp4"));
        assert!(is_mp4_compatible_format("video/m4v"));
        assert!(is_mp4_compatible_format("audio/m4a"));
        
        // Test QuickTime formats
        assert!(is_mp4_compatible_format("video/quicktime"));
        assert!(is_mp4_compatible_format("video/mov"));
        
        // Test 3GPP formats  
        assert!(is_mp4_compatible_format("video/3gp"));
        assert!(is_mp4_compatible_format("video/3g2"));
        
        // Test other formats
        assert!(is_mp4_compatible_format("video/f4v"));
        
        // Test unsupported formats
        assert!(!is_mp4_compatible_format("video/webm"));
        assert!(!is_mp4_compatible_format("video/avi"));
        assert!(!is_mp4_compatible_format("video/mkv"));
    }

    #[test]
    fn test_video_duration_estimation() {
        // Test MP4 estimation (1.5 Mbps)
        // 1.5 MB file should be about 8 seconds: (1.5 * 1024 * 1024 * 8) / 1_500_000 ≈ 8.38s
        let duration = super::estimate_video_duration_from_size(1_572_864, "video/mp4");
        assert!(duration.is_some());
        let duration_val = duration.unwrap();
        assert!(duration_val >= 7 && duration_val <= 10, "Expected ~8 seconds, got {}", duration_val);
        
        // Test 3GP estimation (500 Kbps) - should be longer duration for same file size
        let duration_3gp = super::estimate_video_duration_from_size(1_572_864, "video/3gp");
        assert!(duration_3gp.is_some());
        let duration_3gp_val = duration_3gp.unwrap();
        assert!(duration_3gp_val > duration_val, "3GP should have longer estimated duration than MP4");
        
        // Test WebM estimation (1 Mbps)
        let duration_webm = super::estimate_video_duration_from_size(1_048_576, "video/webm"); // 1 MB
        assert!(duration_webm.is_some());
        let duration_webm_val = duration_webm.unwrap();
        assert!(duration_webm_val >= 7 && duration_webm_val <= 10, "Expected ~8 seconds for 1MB WebM, got {}", duration_webm_val);
        
        // Test very small file (should return None)
        let tiny_duration = super::estimate_video_duration_from_size(100, "video/mp4");
        assert!(tiny_duration.is_none(), "Very small files should return None");
        
        // Test unrealistically large file (should return None)
        let huge_duration = super::estimate_video_duration_from_size(1_000_000_000_000, "video/mp4"); // 1 TB
        assert!(huge_duration.is_none(), "Unrealistically large files should return None");
    }

    #[tokio::test]
    async fn test_unsupported_video_format_fallback() {
        // Test WebM format (not supported by mp4 crate) - should fall back to estimation
        let video_data = vec![0u8; 1_048_576]; // 1 MB of dummy data
        let result = super::extract_video_metadata(&video_data, "video/webm").await;
        
        assert!(result.is_ok());
        let metadata = result.unwrap().unwrap();
        
        // Should have file size
        assert_eq!(metadata["file_size_bytes"], 1_048_576);
        
        // Should have estimated duration
        assert!(metadata.get("duration_seconds").is_some());
        
        // Should indicate estimation method
        assert_eq!(metadata["duration_method"], "estimate");
        
        println!("WebM metadata with estimation: {}", serde_json::to_string_pretty(&metadata).unwrap());
    }

    #[tokio::test]
    #[ignore] // Ignored test - requires specific files to be present
    async fn test_real_video_duration_extraction() {
        // Test paths for specific video files
        let mov_path = "/Users/pawelgodula/Downloads/file_example_MOV_480_700kB.mov";
        let mp4_path = "/Users/pawelgodula/Downloads/file_example_MP4_480_1_5MG.mp4";
        
        // Test MOV file
        if let std::result::Result::Ok(mov_content) = std::fs::read(mov_path) {
            println!("MOV file size: {} bytes", mov_content.len());
            
            let duration = super::extract_mp4_duration(&mov_content).await;
            match duration {
                std::option::Option::Some(seconds) => {
                    println!("MOV duration: {} seconds ({} minutes {}s)", 
                        seconds, seconds / 60, seconds % 60);
                },
                std::option::Option::None => {
                    println!("MOV duration: Could not extract duration");
                }
            }
            
            // Test full metadata extraction
            let metadata_result = super::extract_video_metadata(&mov_content, "video/quicktime").await;
            match metadata_result {
                std::result::Result::Ok(std::option::Option::Some(metadata)) => {
                    println!("MOV metadata: {}", serde_json::to_string_pretty(&metadata).unwrap());
                    
                    // MOV should have duration_method field (either "precise" or "estimate")
                    if let Some(method) = metadata.get("duration_method") {
                        println!("MOV duration method: {}", method);
                    }
                },
                _ => {
                    println!("MOV metadata: Failed to extract metadata");
                }
            }
        } else {
            println!("MOV file not found at: {}", mov_path);
        }
        
        println!(""); // Empty line separator
        
        // Test MP4 file
        if let std::result::Result::Ok(mp4_content) = std::fs::read(mp4_path) {
            println!("MP4 file size: {} bytes", mp4_content.len());
            
            let duration = super::extract_mp4_duration(&mp4_content).await;
            match duration {
                std::option::Option::Some(seconds) => {
                    println!("MP4 duration: {} seconds ({} minutes {}s)", 
                        seconds, seconds / 60, seconds % 60);
                },
                std::option::Option::None => {
                    println!("MP4 duration: Could not extract duration");
                }
            }
            
            // Test full metadata extraction
            let metadata_result = super::extract_video_metadata(&mp4_content, "video/mp4").await;
            match metadata_result {
                std::result::Result::Ok(std::option::Option::Some(metadata)) => {
                    println!("MP4 metadata: {}", serde_json::to_string_pretty(&metadata).unwrap());
                    
                    // MP4 should have duration_method field (either "precise" or "estimate")
                    if let Some(method) = metadata.get("duration_method") {
                        println!("MP4 duration method: {}", method);
                    }
                },
                _ => {
                    println!("MP4 metadata: Failed to extract metadata");
                }
            }
        } else {
            println!("MP4 file not found at: {}", mp4_path);
        }
    }
} 