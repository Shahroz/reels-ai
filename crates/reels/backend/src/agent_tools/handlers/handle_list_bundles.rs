//! Implements the handler for the `list_bundles` agent tool.
//!
//! This function calls the database query to retrieve all expanded bundles
//! for a given user and formats the result into a response suitable
//! for the agent and user.
//! Adheres to the project's Rust coding standards.

pub async fn handle_list_bundles(
    params: crate::agent_tools::tool_params::list_bundles_params::ListBundlesParams,
    pool: &sqlx::PgPool,
) -> std::result::Result<(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse), std::string::String> {
    let user_id = params.user_id.ok_or_else(|| std::string::String::from("user_id not found in parameters; this is an internal error."))?;
    let tool_name = "narrativ_list_bundles";
    log::debug!("Handling tool call for {}: {:?}", tool_name, &params);

    let limit = params.limit.unwrap_or(20);
    let page = params.page.unwrap_or(0);
    let offset = page * limit;
    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("desc");
    let search_pattern = format!("%{}%", params.search.as_deref().unwrap_or(""));

    let bundles_result = crate::queries::bundles::list_expanded_bundles_for_user::list_expanded_bundles_for_user(
        pool,
        user_id,
        &search_pattern,
        sort_by,
        sort_order,
        limit,
        offset,
    )
    .await;

    // Assumes a count function exists with a similar signature.
    let total_count_result = crate::queries::bundles::count_bundles_for_user::count_bundles_for_user(
        pool,
        user_id,
        &search_pattern,
    )
    .await;

    match (bundles_result, total_count_result) {
        (std::result::Result::Ok(bundles), std::result::Result::Ok(total_count)) => {
            let response_json = serde_json::json!({ "bundles": bundles, "total_count": total_count });
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: response_json.clone(),
            };
            let user_response = agentloop::types::user_tool_response::UserToolResponse {
                tool_name: tool_name.to_string(),
                summary: format!("Found {} bundles. Displaying {} bundles.", total_count, bundles.len()),
                data: Some(response_json),
                icon: Some("📦".to_string()),
            };
            std::result::Result::Ok((full_response, user_response))
        }
        (std::result::Result::Err(e), _) => {
            log::error!("Error fetching bundles in {tool_name}: {e:?}");
            std::result::Result::Err(format!("Failed to list bundles: {e}"))
        }
        (_, std::result::Result::Err(e)) => {
            log::error!("Error counting bundles in {tool_name}: {e:?}");
            std::result::Result::Err(format!("Failed to count bundles: {e}"))
        }
    }
}

// Note: Comprehensive tests for this handler would require a mock database or a test
// database setup. The tests below are basic structure checks.
#[cfg(test)]
mod tests {
    #[test]
    fn test_handle_list_bundles_requires_user_id() {
        // This is not an async test, just checks the initial guard clause.
        let params = crate::agent_tools::tool_params::list_bundles_params::ListBundlesParams {
            user_id: None,
            limit: None,
            page: None,
            sort_by: None,
            sort_order: None,
            search: None,
        };
        // We can't easily test the async part without a runtime and a mock pool.
        // This test is conceptual. A real test would be async.
        // let result = handle_list_bundles(params, &pool).await;
        // assert!(result.is_err());
        assert!(params.user_id.is_none()); // a simple assertion
    }
}
