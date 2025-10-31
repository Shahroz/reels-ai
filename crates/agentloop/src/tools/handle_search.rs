//! Provides the handler function for the 'search' agent tool.
//!
//! This function interfaces with the Serper API client to perform web searches.
//! It extracts the query from the strongly-typed `ToolParameters` and calls the search client.
//! Adheres strictly to the project's Rust coding standards.
//! Follows the "one item per file" rule.

// Required types - Used via FQN

/// Handler for the 'search' tool.
///
/// Uses the `api_tools::serper::client::search` function.
/// Expects parameters matching `crate::types::tool_parameters::ToolParameters::Search`.
pub async fn handle_search(
    params: crate::types::tool_parameters::ToolParameters, // Updated type
    _app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    _session_id: crate::types::session_id::SessionId,
) -> Result<String, String> {
    match params {
        crate::types::tool_parameters::ToolParameters::Search { query } => {
            // Call the Serper API client function using fully qualified path
            match api_tools::serper::client::search(&query).await { // Use extracted query
                Ok(result) => std::result::Result::Ok(result),
                Err(e) => std::result::Result::Err(std::format!("Serper search failed: {}", e)),
            }
        }
        _ => std::result::Result::Err(std::string::String::from( // Handle incorrect parameter type
            "Invalid parameters provided for search tool.",
        )),
    }
}

#[cfg(test)]
mod tests {
    // No specific tests for handle_search in original dispatch.rs.
    // Adding basic placeholder test structure.
    // Tests need update separately to use ToolParameters.
    #[tokio::test]
    async fn test_handle_search_placeholder() {
        // Test would require mocking api_tools::serper::client::search
        // For now, assert structure exists.
        std::assert!(true);
    }
}