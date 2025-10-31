//! Defines the payload to start a new research session, including optional attachments.
//!
//! This structure encapsulates the user's initial instruction and any associated
//! attachments (e.g., documents, data files) which initiate the agent loop process.
//! It is used in the API request to begin a session.
//! Conforms to the AgentLoop data structure specifications.

use serde::{Deserialize, Serialize}; // Use for derive macros is common.

/// Payload to start a new research session.
#[derive(schemars::JsonSchema, utoipa::ToSchema, Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRequest {
    pub user_id: uuid::Uuid,
    /// The user's instruction for the research task.
    pub instruction: std::string::String,
    /// Optional list of attachments provided with the research request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: std::option::Option<std::vec::Vec<crate::types::attachment::Attachment>>,
    /// Optional organization ID to use organization credits instead of user credits
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: std::option::Option<uuid::Uuid>,
}

#[cfg(test)]
mod tests {
    // Using super::* for the item under test (ResearchRequest) is idiomatic.
    // Other types should use fully qualified paths.

    #[test]
    fn test_research_request_serde_empty_attachments() {
        let req_no_attachments = super::ResearchRequest {
            user_id: Default::default(),
            instruction: std::string::String::from("Test instruction with no attachments"),
            attachments: None,
            organization_id: None,
        };

        let serialized = serde_json::to_string(&req_no_attachments).unwrap();
        // Check that "attachments" field is not present in JSON when None due to skip_serializing_if
        std::assert!(!serialized.contains("attachments"), "Serialized JSON should not contain 'attachments' field when it's None");

        let deserialized: super::ResearchRequest = serde_json::from_str(&serialized).unwrap();
        std::assert_eq!(deserialized.instruction, "Test instruction with no attachments");
        std::assert!(deserialized.attachments.is_none(), "Deserialized attachments should be None");
    }

    #[test]
    fn test_research_request_serde_with_attachments() {
        // Prepare a sample text attachment
        let text_attachment_data = crate::types::text_attachment::TextAttachment {
            content: std::string::String::from("This is the content of the text attachment."),
        };
        let attachment1 = crate::types::attachment::Attachment {
            title: Some(std::string::String::from("Text Document 1")),
            kind: crate::types::attachment_type::AttachmentType::Text(text_attachment_data.clone()),
        };

        // Prepare a sample PDF attachment (assuming PdfAttachment structure)
        let pdf_attachment_data = crate::types::pdf_attachment::PdfAttachment {
            data: std::vec![0x25, 0x50, 0x44, 0x46], // Minimal "%PDF"
            filename: Some(std::string::String::from("sample_document.pdf")),
        };
        let attachment2 = crate::types::attachment::Attachment {
            title: Some(std::string::String::from("PDF Document 2")),
            kind: crate::types::attachment_type::AttachmentType::Pdf(pdf_attachment_data.clone()),
        };

        let req_with_attachments = super::ResearchRequest {
            user_id: Default::default(),
            instruction: std::string::String::from("Test instruction with multiple attachments"),
            attachments: Some(std::vec![attachment1.clone(), attachment2.clone()]),
            organization_id: None,
        };

        let serialized = serde_json::to_string(&req_with_attachments).unwrap();
        std::assert!(serialized.contains("attachments"), "Serialized JSON should contain 'attachments' field");

        let deserialized: super::ResearchRequest = serde_json::from_str(&serialized).unwrap();
        std::assert_eq!(deserialized.instruction, "Test instruction with multiple attachments");
        std::assert!(deserialized.attachments.is_some(), "Deserialized attachments should be Some");

        let attachments_vec = deserialized.attachments.as_ref().unwrap();
        std::assert_eq!(attachments_vec.len(), 2, "Deserialized attachments count mismatch");

        // Validate first attachment (Text)
        std::assert_eq!(attachments_vec[0].title, attachment1.title, "Attachment 1 title mismatch");
        match &attachments_vec[0].kind {
            crate::types::attachment_type::AttachmentType::Text(text_kind_deserialized) => {
                std::assert_eq!(text_kind_deserialized.content, text_attachment_data.content, "Attachment 1 content mismatch");
            }
            _ => std::panic!("Deserialized attachment 1 is not Text kind"),
        }

        // Validate second attachment (Pdf)
        std::assert_eq!(attachments_vec[1].title, attachment2.title, "Attachment 2 title mismatch");
         match &attachments_vec[1].kind {
            crate::types::attachment_type::AttachmentType::Pdf(pdf_kind_deserialized) => {
                std::assert_eq!(pdf_kind_deserialized.data, pdf_attachment_data.data, "Attachment 2 PDF data mismatch");
                std::assert_eq!(pdf_kind_deserialized.filename, pdf_attachment_data.filename, "Attachment 2 PDF filename mismatch");
            }
            _ => std::panic!("Deserialized attachment 2 is not Pdf kind"),
        }
    }
}