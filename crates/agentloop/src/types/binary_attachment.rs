//! Defines the structure for a generic binary attachment.
//!
//! This struct represents an attachment containing arbitrary binary data,
//! an optional filename, and an optional MIME type (e.g., "application/octet-stream").
//! It serves as a catch-all for unspecified binary file types.
//! Conforms to coding standards requiring one item per file and fully qualified paths.

use schemars::JsonSchema;

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, JsonSchema, PartialEq, Eq)]
pub struct BinaryAttachment {
    pub data: std::vec::Vec<u8>,
    pub filename: std::option::Option<std::string::String>,
    pub mime_type: std::option::Option<std::string::String>,
}

#[cfg(test)]
mod tests {
    // Use super::BinaryAttachment for the item in this file.
    // Fully qualified paths for other items.

    #[test]
    fn test_binary_attachment_creation_all_fields() {
        let data_val = std::vec![0xCA, 0xFE, 0xBA, 0xBE]; // Arbitrary bytes
        let filename_val = std::string::String::from("data.bin");
        let mime_type_val = std::string::String::from("application/octet-stream");
        let attachment = super::BinaryAttachment {
            data: data_val.clone(),
            filename: Some(filename_val.clone()),
            mime_type: Some(mime_type_val.clone()),
        };
        std::assert_eq!(attachment.data, data_val);
        std::assert_eq!(attachment.filename.as_deref(), Some(filename_val.as_str()));
        std::assert_eq!(attachment.mime_type.as_deref(), Some(mime_type_val.as_str()));
    }

    #[test]
    fn test_binary_attachment_creation_optional_fields_none() {
        let data_val = std::vec![0x01, 0x02, 0x03, 0x04];
        let attachment = super::BinaryAttachment {
            data: data_val.clone(),
            filename: None,
            mime_type: None,
        };
        std::assert_eq!(attachment.data, data_val);
        std::assert!(attachment.filename.is_none());
        std::assert!(attachment.mime_type.is_none());
    }

    #[test]
    fn test_binary_attachment_serde_all_fields() {
        let attachment = super::BinaryAttachment {
            data: std::vec![10, 20, 30],
            filename: Some(std::string::String::from("archive.zip")),
            mime_type: Some(std::string::String::from("application/zip")),
        };
        let serialized = serde_json::to_string(&attachment).expect("Serialization failed");
        let deserialized: super::BinaryAttachment = serde_json::from_str(&serialized).expect("Deserialization failed");
        std::assert_eq!(deserialized.data, attachment.data);
        std::assert_eq!(deserialized.filename, attachment.filename);
        std::assert_eq!(deserialized.mime_type, attachment.mime_type);
    }

    #[test]
    fn test_binary_attachment_serde_optional_fields_none() {
        let attachment = super::BinaryAttachment {
            data: std::vec![40, 50, 60],
            filename: None,
            mime_type: None,
        };
        let serialized = serde_json::to_string(&attachment).expect("Serialization failed");
        std::assert!(serialized.contains("\"filename\":null"));
        std::assert!(serialized.contains("\"mime_type\":null"));
        let deserialized: super::BinaryAttachment = serde_json::from_str(&serialized).expect("Deserialization failed");
        std::assert_eq!(deserialized.data, attachment.data);
        std::assert!(deserialized.filename.is_none());
        std::assert!(deserialized.mime_type.is_none());
    }
}