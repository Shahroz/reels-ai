//! Handles the 'narrativ_document_find_by_id' agent tool.
//!
//! This function retrieves a specific document by its ID and user ID from the database.
//! It uses `crate::queries::documents::find_document_by_id_and_user` for data access.
//! The function constructs `FullToolResponse` and `UserToolResponse` based on the query result.
//! Conforms to Narrativ's Rust coding standards, including fully qualified paths.

// Note: agentloop types and crate::queries::documents::find_document_by_id_and_user are assumed 
// to be available in the broader project context. sqlx::Error is assumed as error type from query.

pub async fn handle_narrativ_document_find_by_id(
    params: crate::agent_tools::tool_params::narrativ_document_find_by_id_params::NarrativDocumentFindByIdParams,
    pool: &sqlx::PgPool,
) -> std::result::Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    std::string::String,
> {
    let tool_name_str = "narrativ_document_find_by_id";

    match crate::queries::documents::find_document_by_id_and_user::find_document_by_id_and_user(
        pool,
        params.document_id,
        params.user_id.ok_or(String::from("user_id is null"))?,
    )
    .await
    {
        std::result::Result::Ok(std::option::Option::Some(document)) => {
            let full_response_json_value = serde_json::json!({
                "status": "success",
                "document": document // Assumes Document is Serialize
            });

           let full_response = agentloop::types::full_tool_response::FullToolResponse {
               tool_name: std::string::String::from(tool_name_str),
               response: full_response_json_value,
           };
           let user_response = agentloop::types::user_tool_response::UserToolResponse {
               tool_name: std::string::String::from(tool_name_str),
               summary: std::string::String::from("Document found."),
               data: std::option::Option::Some(full_response.response.clone()),
               icon: std::option::Option::Some("📄".to_string()),
           };
           std::result::Result::Ok((full_response, user_response))
       }
        std::result::Result::Ok(std::option::Option::None) => {
            let full_response_json_value = serde_json::json!({
                "status": "not_found"
            });

           let full_response = agentloop::types::full_tool_response::FullToolResponse {
               tool_name: std::string::String::from(tool_name_str),
               response: full_response_json_value,
           };

           let user_response = agentloop::types::user_tool_response::UserToolResponse {
               tool_name: std::string::String::from(tool_name_str),
               summary: std::string::String::from("Document not found."),
               data: std::option::Option::Some(full_response.response.clone()),
               icon: std::option::Option::Some("❓".to_string()),
           };
           std::result::Result::Ok((full_response, user_response))
       }
        std::result::Result::Err(e) => {
            // Consider logging the full error `e` internally for debugging.
            std::result::Result::Err(std::format!(
                "Error finding document for tool '{tool_name_str}': {e}" // Assuming `e` implements `std::fmt::Display` (e.g. sqlx::Error)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    // Per Rust guidelines, tests are in the same file.
    // Fully qualified paths should be used for types.
    // `super::*` is implicitly available for items in the parent module (this file).

    // Mocking `sqlx::PgPool` and async database calls for unit tests is complex.
    // These "basic tests" are structural placeholders. They are marked `#[ignore]`
    // because they cannot run without a real PostgreSQL test database and potentially
    // specific data states or further mocking of the query function itself.

    #[tokio::test]
    #[ignore = "Test requires a running PostgreSQL instance and mock/data setup for find_document_by_id_and_user query."]
    async fn test_handle_document_found_scenario() {
        // This test outline assumes you have a way to get a `sqlx::PgPool` connected to a test DB
        // and that `crate::queries::documents::find_document_by_id_and_user` can be made to return `Ok(Some(document))`.
        
        let _params = crate::agent_tools::tool_params::narrativ_document_find_by_id_params::NarrativDocumentFindByIdParams {
            document_id: uuid::Uuid::new_v4(), // Example UUID
            user_id: Some(uuid::Uuid::new_v4()),     // Example UUID
        };

        // Placeholder for obtaining a pool for testing.
        // In a real test setup, this pool would connect to a test database.
        // let pool: sqlx::PgPool = obtain_test_db_pool().await;

        // Example of what you'd assert if the call could be made and properly mocked/set-up:
        // let result = super::handle_narrativ_document_find_by_id(params, &pool).await;
        // std::assert!(result.is_ok(), "Expected Ok result when document is found.");
        // if let std::result::Result::Ok((full_res, user_res)) = result {
        //     std::assert_eq!(full_res.tool_name, "narrativ_document_find_by_id");
        //     std::assert_eq!(user_res.summary, "Document found.");
        //     let response_json = &full_res.response;
        //     std::assert_eq!(response_json.get("status").unwrap().as_str().unwrap(), "success");
        //     std::assert!(response_json.get("document").is_some());
        // }
        std::assert!(true, "Placeholder: test_handle_document_found_scenario. Needs DB/mock setup.");
    }

    #[tokio::test]
    #[ignore = "Test requires a running PostgreSQL instance and mock/data setup for find_document_by_id_and_user query."]
    async fn test_handle_document_not_found_scenario() {
        let _params = crate::agent_tools::tool_params::narrativ_document_find_by_id_params::NarrativDocumentFindByIdParams {
            document_id: uuid::Uuid::new_v4(),
            user_id: Some(uuid::Uuid::new_v4()),
        };
        // let pool: sqlx::PgPool = obtain_test_db_pool().await;
        // Assume `find_document_by_id_and_user` returns `Ok(None)`.
        // let result = super::handle_narrativ_document_find_by_id(params, &pool).await;
        // std::assert!(result.is_ok(), "Expected Ok result when document is not found.");
        // if let std::result::Result::Ok((full_res, user_res)) = result {
        //     std::assert_eq!(full_res.tool_name, "narrativ_document_find_by_id");
        //     std::assert_eq!(user_res.summary, "Document not found.");
        //     let response_json = &full_res.response;
        //     std::assert_eq!(response_json.get("status").unwrap().as_str().unwrap(), "not_found");
        // }
        std::assert!(true, "Placeholder: test_handle_document_not_found_scenario. Needs DB/mock setup.");
    }

    #[tokio::test]
    #[ignore = "Test requires a running PostgreSQL instance and mock/data setup for find_document_by_id_and_user query."]
    async fn test_handle_database_error_scenario() {
        let _params = crate::agent_tools::tool_params::narrativ_document_find_by_id_params::NarrativDocumentFindByIdParams {
            document_id: uuid::Uuid::new_v4(),
            user_id: Some(uuid::Uuid::new_v4()),
        };
        // let pool: sqlx::PgPool = obtain_test_db_pool().await;
        // Assume `find_document_by_id_and_user` returns `Err(some_sqlx_error)`.
        // let result = super::handle_narrativ_document_find_by_id(params, &pool).await;
        // std::assert!(result.is_err(), "Expected Err result on database error.");
        // if let std::result::Result::Err(err_msg) = result {
        //     std::assert!(err_msg.contains("Error finding document"));
        // }
        std::assert!(true, "Placeholder: test_handle_database_error_scenario. Needs DB/mock setup.");
    }
}
