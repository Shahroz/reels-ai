//! Defines the structure for a text-based attachment.
//!
//! This struct encapsulates plain text content for an attachment.
//! It is typically used when textual data needs to be associated with a request or entity.
//! Conforms to coding standards requiring one item per file and fully qualified paths.

use schemars::JsonSchema;

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, JsonSchema, PartialEq, Eq)]
pub struct TextAttachment {
    pub content: std::string::String,
}

#[cfg(test)]
mod tests {
    // Use super::TextAttachment for the item in this file.
    // Fully qualified paths for other items.

    #[test]
    fn test_text_attachment_creation_and_fields() {
        let content_val = std::string::String::from("Hello, Rust world!");
        let attachment = super::TextAttachment {
            content: content_val.clone(),
        };
        std::assert_eq!(attachment.content, content_val, "Content field mismatch");
    }

    #[test]
    fn test_text_attachment_serde() {
        let content_val = std::string::String::from("Serializing and Deserializing TextAttachment");
        let attachment = super::TextAttachment {
            content: content_val.clone(),
        };

        let serialized = serde_json::to_string(&attachment)
            .expect("Serialization failed for TextAttachment");

        let deserialized: super::TextAttachment = serde_json::from_str(&serialized)
            .expect("Deserialization failed for TextAttachment");

        std::assert_eq!(deserialized.content, attachment.content, "Deserialized content mismatch");
    }

    #[test]
    fn test_text_attachment_clone_and_debug() {
        let attachment = super::TextAttachment {
            content: std::string::String::from("Test content for clone and debug"),
        };

        let cloned_attachment = attachment.clone();
        std::assert_eq!(cloned_attachment.content, attachment.content, "Cloned content mismatch");

        let debug_str = format!("{:?}", attachment);
        std::assert!(debug_str.contains("TextAttachment"), "Debug string should contain struct name");
        std::assert!(debug_str.contains("Test content for clone and debug"), "Debug string should contain content value");
    }
}