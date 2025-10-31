//! Defines the structure for an image attachment.
//!
//! This struct represents an attachment containing image data (as a byte vector),
//! an optional filename, and an optional MIME type (e.g., "image/png").
//! It is used for handling image files.
//! Conforms to coding standards requiring one item per file and fully qualified paths.

use schemars::JsonSchema;

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, JsonSchema, PartialEq, Eq)]
pub struct ImageAttachment {
    pub data: std::vec::Vec<u8>,
    pub filename: std::option::Option<std::string::String>,
    pub mime_type: std::option::Option<std::string::String>,
}

#[cfg(test)]
mod tests {
    // Use super::ImageAttachment for the item in this file.
    // Fully qualified paths for other items.

    #[test]
    fn test_image_attachment_creation_all_fields() {
        let data_val = std::vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
        let filename_val = std::string::String::from("photo.png");
        let mime_type_val = std::string::String::from("image/png");
        let attachment = super::ImageAttachment {
            data: data_val.clone(),
            filename: Some(filename_val.clone()),
            mime_type: Some(mime_type_val.clone()),
        };
        std::assert_eq!(attachment.data, data_val);
        std::assert_eq!(attachment.filename.as_deref(), Some(filename_val.as_str()));
        std::assert_eq!(attachment.mime_type.as_deref(), Some(mime_type_val.as_str()));
    }

    #[test]
    fn test_image_attachment_creation_optional_fields_none() {
        let data_val = std::vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
        let attachment = super::ImageAttachment {
            data: data_val.clone(),
            filename: None,
            mime_type: None,
        };
        std::assert_eq!(attachment.data, data_val);
        std::assert!(attachment.filename.is_none());
        std::assert!(attachment.mime_type.is_none());
    }

    #[test]
    fn test_image_attachment_serde_all_fields() {
        let attachment = super::ImageAttachment {
            data: std::vec![1, 2, 3],
            filename: Some(std::string::String::from("img.jpg")),
            mime_type: Some(std::string::String::from("image/jpeg")),
        };
        let serialized = serde_json::to_string(&attachment).expect("Serialization failed");
        let deserialized: super::ImageAttachment = serde_json::from_str(&serialized).expect("Deserialization failed");
        std::assert_eq!(deserialized.data, attachment.data);
        std::assert_eq!(deserialized.filename, attachment.filename);
        std::assert_eq!(deserialized.mime_type, attachment.mime_type);
    }

    #[test]
    fn test_image_attachment_serde_optional_fields_none() {
        let attachment = super::ImageAttachment {
            data: std::vec![4, 5, 6],
            filename: None,
            mime_type: None,
        };
        let serialized = serde_json::to_string(&attachment).expect("Serialization failed");
        // Expected: {"data":[4,5,6],"filename":null,"mime_type":null}
        std::assert!(serialized.contains("\"filename\":null"));
        std::assert!(serialized.contains("\"mime_type\":null"));
        let deserialized: super::ImageAttachment = serde_json::from_str(&serialized).expect("Deserialization failed");
        std::assert_eq!(deserialized.data, attachment.data);
        std::assert!(deserialized.filename.is_none());
        std::assert!(deserialized.mime_type.is_none());
    }
}