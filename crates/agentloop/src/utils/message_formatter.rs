//! Formats a message string by optionally including serialized attachments.
//!
//! This utility function takes a main instruction string and an optional slice
//! of attachments. If attachments are provided and non-empty, they are
//! serialized to JSON and prepended to the instruction within a specific format.
//! Handles cases for no attachments, empty attachments, and JSON serialization failures.
//! Adheres to one-item-per-file and FQN guidelines.

// No `use` statements as per guidelines. All types are fully qualified, except prelude items.

// Note: `String`, `Vec`, `Option`, `format!` are used directly as they are typically in the prelude.
// `crate::types::attachment::Attachment` is a local crate type.
// `serde_json::to_string_pretty` is from an external crate.
// `log::warn!` is from an external crate's macros.

pub fn format_message_with_attachments(
    instruction: &str,
    attachments_opt: Option<&Vec<crate::types::attachment::Attachment>>,
) -> String {
    match attachments_opt {
        Some(attachments_vec) if !attachments_vec.is_empty() => {
            match serde_json::to_string_pretty(attachments_vec) {
                Ok(json_string) => {
                    format!(
                        "<ADDITIONAL_CONTEXT>\n{}\n</ADDITIONAL_CONTEXT>\n\n<MAIN_TASK>\n{}\n</MAIN_TASK>",
                        json_string,
                        instruction
                    )
                }
                Err(e) => {
                    // Assuming `log::warn!` is available as shown in other project files.
                    log::warn!("Failed to serialize attachments to JSON: {}. Using error placeholder.", e);
                    format!(
                        "<ADDITIONAL_CONTEXT>\nError: Could not serialize attachments.\n</ADDITIONAL_CONTEXT>\n\n<MAIN_TASK>\n{}\n</MAIN_TASK>",
                        instruction
                    )
                }
            }
        }
        _ => { // Covers None or Some(empty_vec)
            format!("<MAIN_TASK>\n{}\n</MAIN_TASK>", instruction)
        }
    }
}

#[cfg(test)]
mod tests {
    // Access the function under test via `super::`.
    // Fully qualified paths for other types, except prelude items like `String`, `Vec`, `assert_eq!`, `format!`.

    #[test]
    fn test_no_attachments() {
        let instruction = "Perform this task.";
        let expected_message = format!("<MAIN_TASK>\n{}\n</MAIN_TASK>", instruction);
        let result = super::format_message_with_attachments(instruction, None);
        assert_eq!(result, expected_message);
    }

    #[test]
    fn test_empty_attachments_vector() {
        let instruction = "Analyze this data.";
        // Explicit type for attachments to ensure clarity with FQN for Attachment.
        let attachments: Vec<crate::types::attachment::Attachment> = Vec::new();
        let expected_message = format!("<MAIN_TASK>\n{}\n</MAIN_TASK>", instruction);
        let result = super::format_message_with_attachments(instruction, Some(&attachments));
        assert_eq!(result, expected_message);
    }

    #[test]
    fn test_with_attachments_serialization_success() {
        let instruction = "Review these documents.";
        // Types for attachments are fully qualified as per guidelines.
        // `vec!` macro and `String` are prelude.
        let attachments = vec![
            crate::types::attachment::Attachment {
                title: Some(String::from("Doc 1")),
                kind: crate::types::attachment_type::AttachmentType::Text(
                    crate::types::text_attachment::TextAttachment {
                        content: String::from("Content of doc 1."),
                    },
                ),
            },
            crate::types::attachment::Attachment {
                title: None, // No title example
                kind: crate::types::attachment_type::AttachmentType::Text(
                     crate::types::text_attachment::TextAttachment {
                        content: String::from("Content of doc 2, no title."),
                    },
                ),
            },
        ];

        // `serde_json::to_string_pretty` needs `serde_json` crate.
        let expected_json_attachments = serde_json::to_string_pretty(&attachments)
            .expect("Test setup: Failed to serialize test attachments");

        let expected_message = format!(
            "<ADDITIONAL_CONTEXT>\n{}\n</ADDITIONAL_CONTEXT>\n\n<MAIN_TASK>\n{}\n</MAIN_TASK>",
            expected_json_attachments,
            instruction
        );
        let result = super::format_message_with_attachments(instruction, Some(&attachments));
        assert_eq!(result, expected_message);
    }

    // Note on testing serialization failure:
    // The instruction requested testing the JSON serialization failure path if feasible.
    // Inducing a `serde_json::to_string_pretty` error for `Vec<crate::types::attachment::Attachment>`
    // (where Attachment and its fields derive Serialize) without complex mocking or custom types
    // designed to fail serialization is non-trivial in a simple unit test.
    // The error handling logic (log warning and placeholder text) is present in the function
    // `format_message_with_attachments` as requested, mirroring the original `start_research` handler's logic.
}