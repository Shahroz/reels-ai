//! Defines the structure for an attachment provided with a research request.
//!
//! An attachment includes an optional title and a `kind` field, which specifies
//! the type and content of the attachment using the `AttachmentType` enum.
//! Adheres to one-item-per-file and FQN guidelines.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a file or data attached to a research request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, PartialEq, Eq)]
pub struct Attachment {
    /// An optional title for the attachment (e.g., "Document about X").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<std::string::String>,
    /// The kind of the attachment, including its specific data.
    pub kind: crate::types::attachment_type::AttachmentType,
}

#[cfg(test)]
mod tests {
    // Use fully qualified paths for types, super::Attachment for the item in this file.

    #[test]
    fn test_attachment_creation_and_fields() {
        let text_content = std::string::String::from("This is the content of the document.");
        let attachment = super::Attachment {
            title: Some(std::string::String::from("My Document")),
            kind: crate::types::attachment_type::AttachmentType::Text(
                crate::types::text_attachment::TextAttachment { content: text_content.clone() }
            ),
        };

        std::assert_eq!(attachment.title.as_deref(), Some("My Document"), "Title mismatch");
        match &attachment.kind {
            crate::types::attachment_type::AttachmentType::Text(text_attachment) => {
                std::assert_eq!(text_attachment.content, text_content, "Text content mismatch");
            }
            _ => std::panic!("Unexpected attachment kind, expected Text"),
        }

        let cloned_attachment = attachment.clone();
        std::assert_eq!(cloned_attachment.title.as_deref(), Some("My Document"), "Cloned title mismatch");
        match &cloned_attachment.kind {
            crate::types::attachment_type::AttachmentType::Text(text_attachment) => {
                std::assert_eq!(text_attachment.content, text_content, "Cloned text content mismatch");
            }
            _ => std::panic!("Unexpected cloned attachment kind, expected Text"),
        }

        let debug_fmt = format!("{:?}", attachment);
        std::assert!(debug_fmt.contains("My Document"), "Debug output missing title");
        std::assert!(debug_fmt.contains(&text_content), "Debug output missing content");
        std::assert!(debug_fmt.contains("Text(TextAttachment"), "Debug output missing Text kind indicator");
    }

    #[test]
    fn test_attachment_no_title() {
        let pdf_data = std::vec![0x25, 0x50, 0x44, 0x46]; // %PDF
        let pdf_filename = std::string::String::from("document.pdf");
        let attachment = super::Attachment {
            title: None,
            kind: crate::types::attachment_type::AttachmentType::Pdf(
                crate::types::pdf_attachment::PdfAttachment {
                    data: pdf_data.clone(),
                    filename: Some(pdf_filename.clone()),
                }
            ),
        };
        std::assert!(attachment.title.is_none(), "Title should be None");
        match &attachment.kind {
            crate::types::attachment_type::AttachmentType::Pdf(pdf_attachment) => {
                std::assert_eq!(pdf_attachment.data, pdf_data, "PDF data mismatch");
                std::assert_eq!(pdf_attachment.filename.as_deref(), Some(pdf_filename.as_str()), "PDF filename mismatch");
            }
            _ => std::panic!("Unexpected attachment kind, expected Pdf"),
        }
    }

    #[test]
    fn test_attachment_serialization_deserialization() {
        let text_content_val = std::string::String::from("Test Content");
        let attachment = super::Attachment {
            title: Some(std::string::String::from("Test Title")),
            kind: crate::types::attachment_type::AttachmentType::Text(
                crate::types::text_attachment::TextAttachment { content: text_content_val.clone() }
            ),
        };
        let serialized = serde_json::to_string(&attachment).expect("Serialization failed");
        let deserialized: super::Attachment = serde_json::from_str(&serialized).expect("Deserialization failed");

        std::assert_eq!(deserialized.title, attachment.title, "Deserialized title mismatch");
        // Compare kind by serializing both or matching inner content
        let serialized_kind_original = serde_json::to_string(&attachment.kind).unwrap();
        let serialized_kind_deserialized = serde_json::to_string(&deserialized.kind).unwrap();
        std::assert_eq!(serialized_kind_deserialized, serialized_kind_original, "Deserialized kind mismatch");

        // Test with no title
        let image_data_val = std::vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
        let image_filename_val = std::string::String::from("photo.png");
        let image_mime_val = std::string::String::from("image/png");
        let attachment_no_title = super::Attachment {
            title: None,
            kind: crate::types::attachment_type::AttachmentType::Image(
                crate::types::image_attachment::ImageAttachment {
                    data: image_data_val.clone(),
                    filename: Some(image_filename_val.clone()),
                    mime_type: Some(image_mime_val.clone()),
                }
            ),
        };
        let serialized_no_title = serde_json::to_string(&attachment_no_title).expect("Serialization failed");
        // Ensure title is not serialized when None due to skip_serializing_if
        std::assert!(!serialized_no_title.contains("\"title\":"), "Title should not be present in JSON for None");
        let deserialized_no_title: super::Attachment = serde_json::from_str(&serialized_no_title).expect("Deserialization failed");
        std::assert!(deserialized_no_title.title.is_none(), "Deserialized no-title attachment should have None title");
        let serialized_kind_no_title_original = serde_json::to_string(&attachment_no_title.kind).unwrap();
        let serialized_kind_no_title_deserialized = serde_json::to_string(&deserialized_no_title.kind).unwrap();
        std::assert_eq!(serialized_kind_no_title_deserialized, serialized_kind_no_title_original, "Deserialized no-title kind mismatch");
    }
}