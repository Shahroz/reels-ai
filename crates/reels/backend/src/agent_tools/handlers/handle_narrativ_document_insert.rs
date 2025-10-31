//! Handles the insertion of a new document via the 'narrativ_document_insert' tool.
//!
//! This function processes parameters to insert a document into the Narrativ document store.
//! It calls a database query function to perform the actual insertion.
//! On success, it returns structured `FullToolResponse` and `UserToolResponse` objects.
//! Errors encountered during the database operation are mapped to a descriptive string.

pub async fn handle_narrativ_document_insert(
    params: crate::agent_tools::tool_params::narrativ_document_insert_params::NarrativDocumentInsertParams,
    pool: &sqlx::PgPool,
) -> Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    String,
> {
    let tool_name = "narrativ_document_insert";

    let sources_vec: Vec<String> = params.sources.unwrap_or_default();

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

          match crate::queries::documents::insert_document_entry::insert_document_entry(
        &mut tx,
        params.user_id,
        &params.title,
        &params.content,
        &sources_vec,
        params.is_public.unwrap_or(false),
        params.is_task.unwrap_or(false),
        params.include_research,
        std::option::Option::None, // No collection_id for agent-created documents
    )
    .await
    {
        Ok(document) => {
            // Assuming `document` (now InsertedDocumentData) is serde::Serialize
            // and has a `title` field.
            let full_response_output = serde_json::json!({
                "status": "success",
                "document": document
            });

           let ftr = agentloop::types::full_tool_response::FullToolResponse {
               tool_name: String::from(tool_name),
               response: full_response_output,
           };

           let utr = agentloop::types::user_tool_response::UserToolResponse {
               tool_name: std::string::String::from(tool_name),
               summary: std::format!("Document '{}' inserted.", document.title),
               data: std::option::Option::Some(ftr.response.clone()),
               icon: std::option::Option::Some("➕".to_string()),
           };

            tx.commit().await.map_err(|e| e.to_string())?;

            Ok((ftr, utr))
        }
        Err(e) => {
            // tx will rollback on drop
            Err(format!(
                "Failed to insert document '{}': {}",
                params.title, e
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    // Access item under test via `super::`. Fully qualify other paths.
    // These tests are integration tests and require a running PostgreSQL database
    // accessible via DATABASE_URL environment variable for `sqlx::PgPool::connect`.

    // Helper to create example parameters for tests
    fn test_params_valid() -> crate::agent_tools::tool_params::narrativ_document_insert_params::NarrativDocumentInsertParams {
        crate::agent_tools::tool_params::narrativ_document_insert_params::NarrativDocumentInsertParams {
            user_id: Some(uuid::Uuid::new_v4()),
            title: String::from("Test Document Title"),
            content: String::from("Test document content for handler testing."),
            sources: Some(vec![String::from("http://example.com/test_source")]),
            is_public: Some(false),
            is_task: Some(true),
            include_research: Some(
                crate::db::document_research_usage::DocumentResearchUsage::TaskDependent,
            ),
        }
    }

    // Helper function to get a test pool.
    // In a real setup, this would connect to a dedicated test database.
    // Panics if DATABASE_URL is not set or connection fails.
    async fn get_test_db_pool() -> sqlx::PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL environment variable must be set for integration tests");
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    #[ignore] // Ignored by default: Requires a live test database and `DATABASE_URL` env var.
    async fn test_handle_insert_document_success() {
        let params = test_params_valid();
        let pool = get_test_db_pool().await;

        // Ensure the document doesn't already exist if title needs to be unique / for cleanup
        // (Cleanup logic depends on actual DB schema and test strategy)

        let result = super::handle_narrativ_document_insert(params, &pool).await;

        match result {
            Ok((ftr, utr)) => {
                assert_eq!(ftr.tool_name, "narrativ_document_insert");
                let response_json = &ftr.response;
                assert_eq!(response_json.get("status").unwrap().as_str().unwrap(), "success");
                assert!(response_json.get("document").is_some(), "Document field missing in FTR response");
                let doc_json = response_json.get("document").unwrap();
                assert_eq!(
                    doc_json.get("title").unwrap().as_str().unwrap(),
                    "Test Document Title"
                );

                assert_eq!(utr.tool_name, "narrativ_document_insert");
                assert_eq!(utr.summary, "Document 'Test Document Title' inserted.");
                assert!(utr.data.is_some(), "UserToolResponse data should be Some");
                assert_eq!(utr.data.as_ref().unwrap(), response_json, "UserToolResponse data should match FullToolResponse response");
                assert_eq!(utr.icon, std::option::Option::Some("➕".to_string()));

                // Optional: Clean up the inserted document from the database
                // let inserted_doc_id = doc_json.get("id").unwrap().as_str().unwrap();
                // sqlx::query("DELETE FROM documents WHERE id = $1")
                //     .bind(uuid::Uuid::parse_str(inserted_doc_id).unwrap())
                //     .execute(&pool).await.expect("Failed to clean up test document");

            }
            Err(e) => {
                panic!("Test expected success, but got error: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Ignored by default: Requires a live test database and a way to induce a specific query failure.
    async fn test_handle_insert_document_error_path() {
        // This test aims to check the error formatting when `insert_document_entry` fails.
        // Inducing a specific, non-connection related `sqlx::Error` can be complex.
        // One way is to violate a DB constraint if known (e.g., unique constraint).
        // For this example, we'll use params that might lead to an error, or assume a general failure.

        // Using an invalid UUID or data that violates a constraint might trigger an error.
        // However, the most straightforward controlled failure is often a connection issue,
        // but `get_test_db_pool` would panic earlier.

        // This test is more conceptual for error mapping if the query itself fails.
        // If `insert_document_entry` fails (e.g. DB error), the handler should format the error string.
        let params_for_error = crate::agent_tools::tool_params::narrativ_document_insert_params::NarrativDocumentInsertParams {
            user_id: Some(uuid::Uuid::nil()), // Potentially invalid user_id if FK constraint exists and nil is not allowed
            title: String::from("Error Path Test Document"),
            content: String::from("Content for error path test."),
            sources: None,
            is_public: Some(true),
            is_task: Some(false),
            include_research: None,
        };
        let pool = get_test_db_pool().await;

        let result = super::handle_narrativ_document_insert(params_for_error, &pool).await;

        match result {
            Ok(_) => {
                // Clean up if a document was unexpectedly created, then panic.
                // sqlx::query("DELETE FROM documents WHERE title = $1")
                //     .bind("Error Path Test Document")
                //     .execute(&pool).await.ok(); // best effort cleanup
                panic!("Test expected error, but got success.");
            }
            Err(e) => {
                assert!(e.starts_with("Failed to insert document 'Error Path Test Document':"));
                // The rest of the error message `e` will be the `sqlx::Error` string representation.
            }
        }
    }
}