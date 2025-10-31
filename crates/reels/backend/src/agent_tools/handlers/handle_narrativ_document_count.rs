//! Handles the narrativ_document_count tool invocation.
//!
//! This function counts documents for a given user, optionally matching a search pattern.
//! It interacts with the database via the `count_documents_for_user` query.
//! Returns a tuple of FullToolResponse and UserToolResponse on success, or an error string.
//! Follows the one-item-per-file and fully-qualified-paths guidelines.

pub async fn handle_narrativ_document_count(
    params: crate::agent_tools::tool_params::narrativ_document_count_params::NarrativDocumentCountParams,
    pool: &sqlx::PgPool,
) -> std::result::Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    std::string::String, // For the error type
> {
    let tool_name = "narrativ_document_count";

    // Use params.search_pattern, defaulting to "%" if None.
    // .as_deref() converts Option<String> to Option<&str>
    // .unwrap_or() provides a default if None
    let search_pattern = params.search_pattern.as_deref().unwrap_or("%");

    match crate::queries::documents::count_documents_for_user::count_documents_for_user(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        search_pattern,
    )
    .await
    {
        std::result::Result::Ok(count) => {
            let full_response_json = serde_json::json!({
                "status": "success",
                "count": count
            });
           let full_response = agentloop::types::full_tool_response::FullToolResponse {
               tool_name: String::from(tool_name), // String::from is idiomatic
               response: full_response_json,
           };

           let user_response_summary = std::format!("Found {count} document(s).");
           let user_response = agentloop::types::user_tool_response::UserToolResponse {
                tool_name: std::string::String::from(tool_name),
                summary: user_response_summary,
                data: std::option::Option::Some(full_response.response.clone()),
                icon: std::option::Option::Some("🔢".to_string()),
            };

            std::result::Result::Ok((full_response, user_response))
        }
        std::result::Result::Err(e) => {
            std::result::Result::Err(std::format!("Failed to count documents: {e}"))
        }
    }
}

#[cfg(test)]
mod tests {
    // Per guidelines, tests for a function reside in the same file.
    // `super::` is used to access the item in the parent module (the file scope).

    #[test]
    fn test_placeholder_narrativ_document_count() {
        // This is a placeholder. Real tests would require:
        // 1. Mocking `sqlx::PgPool` and the database interaction.
        // 2. Mocking `crate::queries::documents::count_documents_for_user::count_documents_for_user`.
        //    This might involve conditional compilation or a trait-based approach for testability.
        // 3. Constructing `crate::agent_tools::tool_params::narrativ_document_count_params::NarrativDocumentCountParams`.
        //    Ensure this struct derives Debug for easier assertions if needed.
        // 4. Using a runtime like `tokio::test` for async functions.
        //
        // Example structure for an async test:
        //
        // use uuid::Uuid; // Assuming Uuid is from the `uuid` crate and used for user_id
        //
        // #[tokio::test]
       // async fn test_successful_count_example() {
       //     // Assume NarrativDocumentCountParams can be constructed
       //     let params = crate::agent_tools::tool_params::narrativ_document_count_params::NarrativDocumentCountParams {
       //         user_id: uuid::Uuid::new_v4(), // Example: replace with actual type if different
       //         search_pattern: Option::Some(String::from("test%")),
       //     };
       //     
        //     // Mocking the pool and the database query function is crucial here.
        //     // For instance, if `count_documents_for_user` was generic or took a trait object:
        //     // let mut mock_pool = /* ... setup mock_pool ... */;
        //     // In a real scenario, you'd mock the database call to return, e.g., Ok(5).
        //     // let result = super::handle_narrativ_document_count(params, &mock_pool).await;
        //     
        //     // assert!(result.is_ok());
       //     // if let std::result::Result::Ok((full_res, user_res)) = result {
       //     //     assert_eq!(full_res.tool_name, "narrativ_document_count");
       //     //     assert_eq!(user_res.summary, "Found 5 document(s).");
       //     //     // Check full_res.response for {"status": "success", "count": 5}
       //     //     if let serde_json::Value::Object(map) = full_res.response {
       //     //         assert_eq!(map.get("status"), Some(&serde_json::json!("success")));
       //     //         assert_eq!(map.get("count"), Some(&serde_json::json!(5)));
       //     //     } else {
       //     //         panic!("response was not an object");
       //     //     }
       //     // }
       // }
        std::assert!(true, "Placeholder test for handle_narrativ_document_count. Implement actual tests.");
    }
}
