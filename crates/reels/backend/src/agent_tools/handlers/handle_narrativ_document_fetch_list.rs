//! Handles the `narrativ_document_fetch_list` agent tool.
//!
//! This function fetches a list of documents for a user based on specified criteria.
//! It allows for searching, pagination, and sorting of documents.
//! Returns a list of documents or an error if the query fails.
//! Adheres to the one-item-per-file and fully-qualified-paths coding standards.

pub async fn handle_narrativ_document_fetch_list(
    params: crate::agent_tools::tool_params::narrativ_document_fetch_list_params::NarrativDocumentFetchListParams,
    pool: &sqlx::PgPool,
) -> Result<
    (agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse),
    String,
> {
    let tool_name = "narrativ_document_fetch_list";

    // Apply defaults if options are not provided
    let search_pattern = params.search_pattern.as_deref().unwrap_or("%");
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("DESC");

    match crate::queries::documents::fetch_documents_for_user::fetch_documents_for_user(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        search_pattern,
        limit,
        offset,
        sort_by,
        sort_order,
    )
    .await
    {
        Result::Ok(documents) => {
            // Assuming Document struct has #[derive(serde::Serialize)]
            let full_response_json_value = serde_json::json!({
                "status": "success",
                "documents": documents
            });

           let full_response = agentloop::types::full_tool_response::FullToolResponse {
               tool_name: String::from(tool_name),
               response: full_response_json_value,
           };

           let user_response_summary = format!("Fetched {} document(s).", documents.len());
           let user_response = agentloop::types::user_tool_response::UserToolResponse { // This was already correct
               tool_name: std::string::String::from(tool_name),
               summary: user_response_summary,
               data: std::option::Option::Some(full_response.response.clone()),
               icon: std::option::Option::Some("📄".to_string()),
           };
           Result::Ok((full_response, user_response))
       }
        Result::Err(e) => {
            Result::Err(format!("Failed to fetch documents: {e}"))
        }
    }
}
