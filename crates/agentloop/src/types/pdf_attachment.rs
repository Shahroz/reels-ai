//! Defines the structure for a PDF attachment.
//!
//! This struct represents an attachment containing PDF data as a byte vector,
//! and an optional filename.
//! It is used for handling PDF documents within the system.
//! Conforms to coding standards requiring one item per file and fully qualified paths.

use schemars::JsonSchema;

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, JsonSchema, PartialEq, Eq)]
pub struct PdfAttachment {
    pub data: std::vec::Vec<u8>,
    pub filename: std::option::Option<std::string::String>,
}

#[cfg(test)]
mod tests {
    // Use super::PdfAttachment for the item in this file.
    // Fully qualified paths for other items.

    #[test]
    fn test_pdf_attachment_creation_with_filename() {
        let data_val = std::vec![0x25, 0x50, 0x44, 0x46]; // Minimal PDF header "%PDF"
        let filename_val = std::string::String::from("document.pdf");
        let attachment = super::PdfAttachment {
            data: data_val.clone(),
            filename: Some(filename_val.clone()),
        };
        std::assert_eq!(attachment.data, data_val);
        std::assert_eq!(attachment.filename.as_deref(), Some(filename_val.as_str()));
    }

    #[test]
    fn test_pdf_attachment_creation_without_filename() {
        let data_val = std::vec![0xDE, 0xAD, 0xBE, 0xEF];
        let attachment = super::PdfAttachment {
            data: data_val.clone(),
            filename: None,
        };
        std::assert_eq!(attachment.data, data_val);
        std::assert!(attachment.filename.is_none());
    }

    #[test]
    fn test_pdf_attachment_serde_with_filename() {
        let attachment = super::PdfAttachment {
            data: std::vec![1, 2, 3],
            filename: Some(std::string::String::from("test.pdf")),
        };
        let serialized = serde_json::to_string(&attachment).expect("Serialization failed");
        let deserialized: super::PdfAttachment = serde_json::from_str(&serialized).expect("Deserialization failed");
        std::assert_eq!(deserialized.data, attachment.data);
        std::assert_eq!(deserialized.filename, attachment.filename);
    }

    #[test]
    fn test_pdf_attachment_serde_without_filename() {
        let attachment = super::PdfAttachment {
            data: std::vec![4, 5, 6],
            filename: None,
        };
        let serialized = serde_json::to_string(&attachment).expect("Serialization failed");
        // Expected: {"data":[4,5,6],"filename":null}
        std::assert!(serialized.contains("\"filename\":null"));
        let deserialized: super::PdfAttachment = serde_json::from_str(&serialized).expect("Deserialization failed");
        std::assert_eq!(deserialized.data, attachment.data);
        std::assert!(deserialized.filename.is_none());
    }
}