//! Defines the structure for a video URL attachment.
//!
//! This struct represents an attachment that is a URL pointing to a video resource.
//! It is used for linking external video content, such as YouTube or Vimeo links.
//! Conforms to coding standards requiring one item per file and fully qualified paths.

use schemars::JsonSchema;

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, JsonSchema, PartialEq, Eq)]
pub struct VideoUrlAttachment {
    pub url: std::string::String,
}

#[cfg(test)]
mod tests {
    // Use super::VideoUrlAttachment for the item in this file.
    // Fully qualified paths for other items.

    #[test]
    fn test_video_url_attachment_creation_and_fields() {
        let url_val = std::string::String::from("https://www.example.com/video.mp4");
        let attachment = super::VideoUrlAttachment {
            url: url_val.clone(),
        };
        std::assert_eq!(attachment.url, url_val, "URL field mismatch");
    }

    #[test]
    fn test_video_url_attachment_serde() {
        let url_val = std::string::String::from("https://example.org/another_video");
        let attachment = super::VideoUrlAttachment {
            url: url_val.clone(),
        };

        let serialized = serde_json::to_string(&attachment)
            .expect("Serialization failed for VideoUrlAttachment");

        let deserialized: super::VideoUrlAttachment = serde_json::from_str(&serialized)
            .expect("Deserialization failed for VideoUrlAttachment");

        std::assert_eq!(deserialized.url, attachment.url, "Deserialized URL mismatch");
    }

    #[test]
    fn test_video_url_attachment_clone_and_debug() {
        let attachment = super::VideoUrlAttachment {
            url: std::string::String::from("https://test.com/video_for_debug"),
        };

        let cloned_attachment = attachment.clone();
        std::assert_eq!(cloned_attachment.url, attachment.url, "Cloned URL mismatch");

        let debug_str = format!("{:?}", attachment);
        std::assert!(debug_str.contains("VideoUrlAttachment"), "Debug string should contain struct name");
        std::assert!(debug_str.contains("https://test.com/video_for_debug"), "Debug string should contain URL value");
    }
}