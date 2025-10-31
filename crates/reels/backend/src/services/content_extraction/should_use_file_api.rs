//! Determines whether a file should use the Gemini File API or inline_data approach.
//!
//! This function provides the routing logic for content extraction. Videos are processed
//! through the File API for better handling of large files and improved transcription
//! capabilities. All other file types continue to use the existing inline_data approach
//! to preserve current functionality and avoid unnecessary changes.

/// Determines whether a file should use the Gemini File API.
///
/// # Arguments
/// * `mime_type` - The MIME type of the file
///
/// # Returns
/// `true` if the file should use the File API, `false` for inline_data approach
pub fn should_use_file_api(mime_type: &str) -> bool {
    // Route all videos through File API for better transcription and large file support
    mime_type.starts_with("video/")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_video_files_use_file_api() {
        assert!(super::should_use_file_api("video/mp4"));
        assert!(super::should_use_file_api("video/mov"));
        assert!(super::should_use_file_api("video/avi"));
        assert!(super::should_use_file_api("video/webm"));
    }

    #[test]
    fn test_non_video_files_use_inline_data() {
        assert!(!super::should_use_file_api("image/jpeg"));
        assert!(!super::should_use_file_api("application/pdf"));
        assert!(!super::should_use_file_api("text/plain"));
        assert!(!super::should_use_file_api("application/vnd.openxmlformats-officedocument.wordprocessingml.document"));
    }

    #[test]
    fn test_empty_mime_type() {
        assert!(!super::should_use_file_api(""));
    }

    #[test]
    fn test_partial_video_match() {
        // Should not match non-video types that contain "video"
        assert!(!super::should_use_file_api("application/video-metadata"));
        assert!(!super::should_use_file_api("text/video-description"));
    }
} 