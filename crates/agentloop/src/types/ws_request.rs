//! Defines the requests sent by clients to the AgentLoop service over WebSocket.
//!
//! This enum encapsulates messages initiated by the client, such as providing
//! user input (which can include attachments for additional context) or
//! requesting an interruption of the ongoing process.
//! Adheres to the one-item-per-file guideline and uses fully qualified paths.

//! Revision History
//! - 2025-05-13T15:18:00Z @AI: Add optional attachments to UserInput variant.
//! - 2025-04-24T12:48:48Z @AI: Initial implementation based on handler assumptions.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema; // Added for OpenAPI schema generation
use schemars::JsonSchema; // Often needed with ToSchema

/// Messages sent by clients to the AgentLoop service via WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)] // Added ToSchema and JsonSchema
pub enum WebsocketRequest {
    /// Represents input or instructions provided by the user during a session.
    UserInput {
        instruction: std::string::String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attachments: std::vec::Vec<crate::types::attachment::Attachment>,
    },
    /// A request from the client to interrupt or pause the current agent process.
    Interrupt,
    // Add other potential client requests here, e.g., Ping, QueryStatus
}

#[cfg(test)]
mod tests {
    // Use super::WebsocketRequest for the item in this file.
    // Use fully qualified paths for other types.

    #[test]
    fn test_user_input_serialization_deserialization() {
        // Case 1: UserInput without attachments
        let user_input_no_attachments = super::WebsocketRequest::UserInput {
            instruction: std::string::String::from("Test instruction without attachments"),
            attachments: std::vec::Vec::new(),
        };

        let serialized_no_attachments = serde_json::to_string(&user_input_no_attachments)
            .expect("Serialization of UserInput without attachments failed");

        // With #[serde(default, skip_serializing_if = "Vec::is_empty")], "attachments" field should be absent
        std::assert!(
            !serialized_no_attachments.contains("attachments"),
            "Serialized JSON should not contain 'attachments' field when empty"
        );

        let deserialized_no_attachments: super::WebsocketRequest =
            serde_json::from_str(&serialized_no_attachments)
                .expect("Deserialization of UserInput without attachments failed");

        match deserialized_no_attachments {
            super::WebsocketRequest::UserInput { instruction, attachments } => {
                std::assert_eq!(
                    instruction,
                    "Test instruction without attachments",
                    "Instruction mismatch after deserialization (no attachments)"
                );
                std::assert!(
                    attachments.is_empty(),
                    "Attachments should be empty after deserialization (no attachments)"
                );
            }
            _ => std::panic!("Deserialized into wrong variant (no attachments case)"),
        }

        // Case 2: UserInput with one attachment
        let text_content = std::string::String::from("Sample attachment content");
        let attachment1 = crate::types::attachment::Attachment {
            title: Some(std::string::String::from("Doc1.txt")),
            kind: crate::types::attachment_type::AttachmentType::Text(
                crate::types::text_attachment::TextAttachment { content: text_content.clone() },
            ),
        };
        let user_input_with_one_attachment = super::WebsocketRequest::UserInput {
            instruction: std::string::String::from("Instruction with one attachment"),
            attachments: std::vec![attachment1.clone()],
        };

        let serialized_with_one_attachment = serde_json::to_string(&user_input_with_one_attachment)
            .expect("Serialization of UserInput with one attachment failed");

        std::assert!(
            serialized_with_one_attachment.contains("attachments"),
            "Serialized JSON should contain 'attachments' field when not empty"
        );

        let deserialized_with_one_attachment: super::WebsocketRequest =
            serde_json::from_str(&serialized_with_one_attachment)
                .expect("Deserialization of UserInput with one attachment failed");

        match deserialized_with_one_attachment {
            super::WebsocketRequest::UserInput { instruction, attachments } => {
                std::assert_eq!(
                    instruction,
                    "Instruction with one attachment",
                    "Instruction mismatch after deserialization (one attachment)"
                );
                std::assert_eq!(attachments.len(), 1, "Attachments count should be 1");
                std::assert_eq!(
                    attachments[0].title.as_deref(),
                    Some("Doc1.txt"),
                    "Attachment title mismatch"
                );
                match &attachments[0].kind {
                    crate::types::attachment_type::AttachmentType::Text(text_attachment) => {
                        std::assert_eq!(text_attachment.content, text_content, "Attachment content mismatch");
                    }
                    _ => std::panic!("Unexpected attachment kind in deserialized data"),
                }
            }
            _ => std::panic!("Deserialized into wrong variant (one attachment case)"),
        }
    }

    #[test]
    fn test_user_input_clone_and_debug() {
        let instruction_text = std::string::String::from("Instruction for clone and debug");
        let attachment_content_text = std::string::String::from("Debug attachment text");
        let attachment_for_test = crate::types::attachment::Attachment {
            title: Some(std::string::String::from("DebugDoc.md")),
            kind: crate::types::attachment_type::AttachmentType::Text(
                crate::types::text_attachment::TextAttachment { content: attachment_content_text.clone() },
            ),
        };

        let original_user_input = super::WebsocketRequest::UserInput {
            instruction: instruction_text.clone(),
            attachments: std::vec![attachment_for_test.clone()],
        };

        // Test Clone
        let cloned_user_input = original_user_input.clone();
        match cloned_user_input {
            super::WebsocketRequest::UserInput { ref instruction, ref attachments } => {
                std::assert_eq!(*instruction, instruction_text, "Cloned instruction mismatch");
                std::assert_eq!(attachments.len(), 1, "Cloned attachments count mismatch");
                std::assert_eq!(attachments[0].title.as_deref(), Some("DebugDoc.md"), "Cloned attachment title mismatch");
                 match &attachments[0].kind {
                    crate::types::attachment_type::AttachmentType::Text(text_attachment) => {
                        std::assert_eq!(text_attachment.content, attachment_content_text, "Cloned attachment content mismatch");
                    }
                    _ => std::panic!("Unexpected cloned attachment kind"),
                }
            }
            _ => std::panic!("Cloned UserInput is not of UserInput variant"),
        }

        // Test Debug
        let debug_output_str = format!("{:?}", original_user_input);
        std::assert!(debug_output_str.contains("UserInput"), "Debug output missing 'UserInput'");
        std::assert!(debug_output_str.contains(&instruction_text), "Debug output missing instruction text");
        std::assert!(debug_output_str.contains("attachments"), "Debug output missing 'attachments' field name");
        std::assert!(debug_output_str.contains("DebugDoc.md"), "Debug output missing attachment title");
        std::assert!(debug_output_str.contains(&attachment_content_text), "Debug output missing attachment content");
    }

    #[test]
    fn test_interrupt_serialization_deserialization_and_derives() {
        let interrupt_request = super::WebsocketRequest::Interrupt;

        // Test Serialize/Deserialize
        let serialized_interrupt = serde_json::to_string(&interrupt_request)
            .expect("Serialization of Interrupt failed");
        std::assert_eq!(serialized_interrupt, "\"Interrupt\"", "Serialized Interrupt JSON mismatch");

        let deserialized_interrupt: super::WebsocketRequest = serde_json::from_str(&serialized_interrupt)
            .expect("Deserialization of Interrupt failed");

        match deserialized_interrupt {
            super::WebsocketRequest::Interrupt => { /* Correct */ }
            _ => std::panic!("Deserialized Interrupt is not of Interrupt variant"),
        }

        // Test Clone
        let cloned_interrupt = interrupt_request.clone();
         match cloned_interrupt {
            super::WebsocketRequest::Interrupt => { /* Correct */ }
            _ => std::panic!("Cloned Interrupt is not of Interrupt variant"),
        }

        // Test Debug
        let debug_output_interrupt = format!("{:?}", interrupt_request);
        std::assert_eq!(debug_output_interrupt, "Interrupt", "Debug output for Interrupt mismatch");
    }
}