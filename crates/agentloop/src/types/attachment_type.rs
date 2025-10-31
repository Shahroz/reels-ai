//! Defines the types of attachments that can be associated with a research request.
//!
//! This enum allows for future expansion to include various file types like PDFs,
//! images, or video URLs, beyond simple text documents.
//! Adheres to one-item-per-file and FQN guidelines.

use schemars::JsonSchema;

/// Enum representing the type of an attachment.
#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, JsonSchema, PartialEq, Eq)]
pub enum AttachmentType {
    /// Plain text content.
    Text(crate::types::text_attachment::TextAttachment),
    /// PDF document (placeholder for future support).
    Pdf(crate::types::pdf_attachment::PdfAttachment),
    /// Image file (placeholder for future support).
    Image(crate::types::image_attachment::ImageAttachment),
    /// URL to a video (e.g., YouTube) (placeholder for future support).
    VideoUrl(crate::types::video_url_attachment::VideoUrlAttachment),
    /// Generic binary file (placeholder for future support).
    Binary(crate::types::binary_attachment::BinaryAttachment),
}

#[cfg(test)]
mod tests {
    // Use fully qualified paths for types.
    // super::AttachmentType is used to refer to the item in this file.

    #[test]
    fn test_attachment_type_derives() {
        let original_text_content = std::string::String::from("Test Content for Derives");
        let original_text_struct = crate::types::text_attachment::TextAttachment {
            content: original_text_content.clone(),
        };
        let text_type_variant = super::AttachmentType::Text(original_text_struct.clone());

        // Test Clone
        let cloned_type_variant = text_type_variant.clone();
        if let super::AttachmentType::Text(inner_cloned) = cloned_type_variant {
            std::assert_eq!(inner_cloned.content, original_text_content, "Cloned content mismatch");
        } else {
            std::panic!("Cloned type is not Text variant after clone");
        }

        // Test Debug
        let debug_fmt = format!("{:?}", text_type_variant);
        std::assert!(debug_fmt.contains("Text(TextAttachment"), "Debug format should indicate Text variant and its struct.");
        std::assert!(debug_fmt.contains(&original_text_content), "Debug format should contain the content.");

        // Test serialization
        let serialized = serde_json::to_string(&text_type_variant).expect("Serialization failed for Text variant in derives test");
        // Expected: {"Text":{"content":"Test Content for Derives"}}
        std::assert!(serialized.contains("\"Text\""), "Serialized JSON should contain Text key.");
        std::assert!(serialized.contains(&format!("\"content\":\"{}\"", original_text_content)), "Serialized JSON should contain content field and value.");

        // Test deserialization
        let deserialized: super::AttachmentType = serde_json::from_str(&serialized).expect("Deserialization failed for Text variant in derives test");
        if let super::AttachmentType::Text(inner_deserialized) = deserialized {
            std::assert_eq!(inner_deserialized.content, original_text_content, "Deserialized content mismatch");
        } else {
            std::panic!("Deserialized type is not Text variant after deserialization");
        }
    }

    #[test]
    fn test_all_attachment_types_serialization_and_deserialization() {
        // Text Variant
        let text_content = std::string::String::from("Sample text content");
        let original_text_struct = crate::types::text_attachment::TextAttachment { content: text_content.clone() };
        let text_variant = super::AttachmentType::Text(original_text_struct);
        let serialized_text = serde_json::to_string(&text_variant).expect("Serialization failed for Text variant");
        let deserialized_text: super::AttachmentType = serde_json::from_str(&serialized_text).expect("Deserialization failed for Text variant");
        match deserialized_text {
            super::AttachmentType::Text(inner) => std::assert_eq!(inner.content, text_content),
            _ => std::panic!("Deserialized type not Text for Text variant test"),
        }

        // Pdf Variant
        let pdf_data = std::vec![0x25, 0x50, 0x44, 0x46]; // %PDF
        let pdf_filename = std::string::String::from("document.pdf");
        let original_pdf_struct = crate::types::pdf_attachment::PdfAttachment { data: pdf_data.clone(), filename: Some(pdf_filename.clone()) };
        let pdf_variant = super::AttachmentType::Pdf(original_pdf_struct);
        let serialized_pdf = serde_json::to_string(&pdf_variant).expect("Serialization failed for Pdf variant");
        let deserialized_pdf: super::AttachmentType = serde_json::from_str(&serialized_pdf).expect("Deserialization failed for Pdf variant");
        match deserialized_pdf {
            super::AttachmentType::Pdf(inner) => {
                std::assert_eq!(inner.data, pdf_data);
                std::assert_eq!(inner.filename.as_deref(), Some(pdf_filename.as_str()));
            }
            _ => std::panic!("Deserialized type not Pdf for Pdf variant test"),
        }

        // Image Variant
        let image_data = std::vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
        let image_filename = std::string::String::from("photo.png");
        let image_mime = std::string::String::from("image/png");
        let original_image_struct = crate::types::image_attachment::ImageAttachment {
            data: image_data.clone(),
            filename: Some(image_filename.clone()),
            mime_type: Some(image_mime.clone()),
        };
        let image_variant = super::AttachmentType::Image(original_image_struct);
        let serialized_image = serde_json::to_string(&image_variant).expect("Serialization failed for Image variant");
        let deserialized_image: super::AttachmentType = serde_json::from_str(&serialized_image).expect("Deserialization failed for Image variant");
        match deserialized_image {
            super::AttachmentType::Image(inner) => {
                std::assert_eq!(inner.data, image_data);
                std::assert_eq!(inner.filename.as_deref(), Some(image_filename.as_str()));
                std::assert_eq!(inner.mime_type.as_deref(), Some(image_mime.as_str()));
            }
            _ => std::panic!("Deserialized type not Image for Image variant test"),
        }

        // VideoUrl Variant
        let video_url_str = std::string::String::from("https://example.com/video.mp4");
        let original_video_struct = crate::types::video_url_attachment::VideoUrlAttachment { url: video_url_str.clone() };
        let video_variant = super::AttachmentType::VideoUrl(original_video_struct);
        let serialized_video = serde_json::to_string(&video_variant).expect("Serialization failed for VideoUrl variant");
        let deserialized_video: super::AttachmentType = serde_json::from_str(&serialized_video).expect("Deserialization failed for VideoUrl variant");
        match deserialized_video {
            super::AttachmentType::VideoUrl(inner) => std::assert_eq!(inner.url, video_url_str),
            _ => std::panic!("Deserialized type not VideoUrl for VideoUrl variant test"),
        }

        // Binary Variant
        let binary_data = std::vec![0x01, 0x02, 0x03, 0x04];
        let binary_filename = std::string::String::from("archive.dat");
        let binary_mime = std::string::String::from("application/octet-stream");
        let original_binary_struct = crate::types::binary_attachment::BinaryAttachment {
            data: binary_data.clone(),
            filename: Some(binary_filename.clone()),
            mime_type: Some(binary_mime.clone()),
        };
        let binary_variant = super::AttachmentType::Binary(original_binary_struct);
        let serialized_binary = serde_json::to_string(&binary_variant).expect("Serialization failed for Binary variant");
        let deserialized_binary: super::AttachmentType = serde_json::from_str(&serialized_binary).expect("Deserialization failed for Binary variant");
        match deserialized_binary {
            super::AttachmentType::Binary(inner) => {
                std::assert_eq!(inner.data, binary_data);
                std::assert_eq!(inner.filename.as_deref(), Some(binary_filename.as_str()));
                std::assert_eq!(inner.mime_type.as_deref(), Some(binary_mime.as_str()));
            }
            _ => std::panic!("Deserialized type not Binary for Binary variant test"),
        }
    }
}