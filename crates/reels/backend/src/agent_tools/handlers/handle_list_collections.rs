//! Handles the 'narrativ_list_collections' agent tool action.
//!
//! This function takes `ListCollectionsParams`, calls the corresponding query functions,
//! and maps the result to agent-compatible `FullToolResponse` and `UserToolResponse`.
//! It adheres to the standard Rust coding guidelines for this project.

use serde_json::json;

pub async fn handle_list_collections(
    params: crate::agent_tools::tool_params::list_collections_params::ListCollectionsParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let user_id = params.user_id.ok_or("The user_id should be provided".to_owned())?;
    let tool_name = "narrativ_list_collections";
    log::debug!("Handling tool call for {tool_name}: {params:?}");

    let limit = params.limit.unwrap_or(20);
    let page = params.page.unwrap_or(0);
    let offset = page * limit;
    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("desc");
    let search_pattern = format!("%{}%", params.search.as_deref().unwrap_or(""));

    let collections_result = crate::queries::collections::list_collections::list_collections(
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
    let total_count_result = crate::queries::collections::count_collections::count_collections(
        pool,
        user_id,
        &search_pattern,
    )
    .await;

    match (collections_result, total_count_result) {
        (Ok(collections), Ok(total_count)) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: json!({ "collections": collections, "total_count": total_count }),
            };
            let user_response = agentloop::types::user_tool_response::UserToolResponse {
                tool_name: tool_name.to_string(),
                summary: format!("Found {} collections. Displaying {} collections.", total_count, collections.len()),
                data: Some(full_response.response.clone()),
                icon: Some("📚".to_string()),
            };
            Ok((full_response, user_response))
        }
        (Err(e), _) => {
            log::error!("Error fetching collections in {tool_name}: {e:?}");
            Err(format!("Failed to list collections: {e}"))
        }
        (_, Err(e)) => {
            log::error!("Error counting collections in {tool_name}: {e:?}");
            Err(format!("Failed to count collections: {e}"))
        }
    }
}