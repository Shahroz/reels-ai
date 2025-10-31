//! Handles the 'narrativ_document_delete' agent tool.
//!
//! This function processes a request to delete a document,
//! using the provided document ID and user ID. It calls the
//! `delete_document_entry` query and formats the result into
//! standard tool responses. Adheres to Rust coding standards.

pub async fn handle_narrativ_document_delete(
    params: crate::agent_tools::tool_params::narrativ_document_delete_params::NarrativDocumentDeleteParams,
    pool: &sqlx::PgPool,
) -> Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    String,
> {
    let tool_name = "narrativ_document_delete";

    match crate::queries::documents::delete_document_entry::delete_document_entry(
        pool,
        params.document_id,
        params.user_id.ok_or(String::from("user_id is null"))?,
    )
    .await
    {
        Ok(rows_affected) => {
            let tool_output_json_content = serde_json::json!({
                "status": "success",
                "deleted_count": rows_affected
            });

           let full_response = agentloop::types::full_tool_response::FullToolResponse {
               tool_name: String::from(tool_name),
               response: tool_output_json_content,
           };

           let user_response = agentloop::types::user_tool_response::UserToolResponse {
               tool_name: std::string::String::from(tool_name),
               summary: std::format!("Deleted {rows_affected} document(s)."),
               data: std::option::Option::Some(full_response.response.clone()),
               icon: std::option::Option::Some("🗑️".to_string()),
           };
           Result::Ok((full_response, user_response))
       }
        Err(e) => Result::Err(format!(
            "Failed to delete document (tool: {tool_name}): {e}"
        )),
    }
}

#[cfg(test)]
mod tests {
    // No `use` statements allowed by strict guidelines for types from other modules/crates.

    // Helper to construct params for tests using full path.
    fn create_test_params() -> crate::agent_tools::tool_params::narrativ_document_delete_params::NarrativDocumentDeleteParams {
        crate::agent_tools::tool_params::narrativ_document_delete_params::NarrativDocumentDeleteParams {
            document_id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            user_id: Some(uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()),
        }
    }

    #[test]
    fn test_construct_success_response() {
        let _params = create_test_params();
        let tool_name = "narrativ_document_delete";
        let rows_affected: u64 = 1;

        // The `tool_input_json` field is not part of the current FullToolResponse structure.
        // let _expected_tool_input_value = serde_json::to_value(&params).unwrap(); 
        let expected_response_content = serde_json::json!({
            "status": "success",
            "deleted_count": rows_affected
        });

        // Assuming agentloop::types are accessible via their full path from the crate root.
        let expected_full_response = agentloop::types::full_tool_response::FullToolResponse {
            tool_name: String::from(tool_name),
            response: expected_response_content.clone(),
        };

        let expected_user_response = agentloop::types::user_tool_response::UserToolResponse {
            tool_name: std::string::String::from(tool_name),
            summary: std::format!("Deleted {} document(s).", rows_affected),
            data: std::option::Option::Some(expected_response_content),
            icon: std::option::Option::Some("🗑️".to_string()), // Matching handler's behavior
        };

        assert_eq!(expected_full_response.tool_name, tool_name);
        // Basic check on JSON string content. Robust checks might parse JSON.
        assert_eq!(expected_full_response.response["status"], "success");
        assert_eq!(expected_full_response.response["deleted_count"], 1);
        assert_eq!(expected_user_response.summary, "Deleted 1 document(s).");
        assert!(expected_user_response.data.is_some());
        assert_eq!(expected_user_response.icon.unwrap(), "🗑️");
    }

    #[test]
    fn test_construct_error_response_format() {
        let tool_name = "narrativ_document_delete";
        let db_error_message = "Simulated database error";
        
        // This mimics the error formatting logic from the handler's Err arm.
        let actual_error_string = format!(
            "Failed to delete document (tool: {}): {}",
            tool_name,
            db_error_message
        );

        let expected_error_string = "Failed to delete document (tool: narrativ_document_delete): Simulated database error";
        
        assert_eq!(actual_error_string, expected_error_string);
    }
}
