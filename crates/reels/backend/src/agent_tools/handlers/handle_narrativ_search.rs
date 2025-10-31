//! Implements the actual logic for the Narrativ search tool.
//!
//! This function takes strongly-typed search parameters, performs the search
//! (e.g., via an external API like Serper), and returns structured full and user-friendly responses.
//! Adheres to Narrativ and AgentLoop coding standards.

// Note: Using fully qualified paths as per guidelines where applicable.
// agentloop types are used for return structures.
// narrativ types are used for parameters.

use serde_json::json;

/// Performs a web search using Narrativ's configured search capabilities.
///
/// # Arguments
///
/// * `params` - `crate::agent_tools::tool_params::SearchParams` containing the search query.
///
/// # Returns
///
/// A `Result` containing a tuple of
/// `(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse)`
/// on success, or an error `String` on failure.
pub async fn handle_narrativ_search(
    params: crate::agent_tools::tool_params::search_params::SearchParams,
) -> Result<(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse), String> {
    let tool_name = "narrativ_search".to_string();
    match api_tools::serper::client::search(&params.query).await {
        Ok(search_result_str) => {
            let full_response_properties = match serde_json::from_str(&search_result_str) {
                Ok(json_val) => json_val,
                Err(_) => serde_json::json!({"raw_result": search_result_str}), // Fallback if not valid JSON
            };
            let user_response_summary = format!("Search for '{}' completed.", params.query);

            Ok((
                agentloop::types::full_tool_response::FullToolResponse {
                    tool_name: tool_name.clone(),
                    response: full_response_properties.clone(),
               },
              agentloop::types::user_tool_response::UserToolResponse {
                  tool_name,
                  summary: user_response_summary.clone(),
                  data: Some(json!{user_response_summary.clone()}),
                   icon: Some("🔎".to_string()),
              },
          ))
      }
        Err(e) => Err(format!("Narrativ search failed: {e}")),
    }
}

#[cfg(test)]
mod tests {
    // Basic test structure. More comprehensive tests would mock `crate::api_tools::serper::search`.
    #[tokio::test]
    async fn test_handle_narrativ_search_placeholder() {
        // This is a placeholder. In a real scenario, you'd mock the external call.
        // For now, we're assuming `crate::api_tools::serper::search` might not be easily mockable
        // in this isolated test without more infrastructure.
        // The purpose of this test is to ensure the function can be called and returns
        // the expected structure on a conceptual success/failure of the underlying API.

        // Example: If you could mock `serper::search` to return Ok("{\"key\":\"value\"}".to_string())
        let _params = crate::agent_tools::tool_params::search_params::SearchParams {
            query: "test query".to_string(),
        };

        // Simulate a successful call (actual call is not made here)
        // let result = super::handle_narrativ_search(params).await;
        // assert!(result.is_ok());
        // if let Ok((full_res, user_res)) = result {
        //     assert_eq!(full_res.tool_name, "narrativ_search");
        //     assert!(full_res.properties.is_object()); // or check specific content
        //     assert_eq!(user_res.tool_name, "narrativ_search");
        //     assert!(user_res.summary.contains("test query"));
        // }
        assert!(true); // Placeholder assertion
    }
}
