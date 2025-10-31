//! Handles the 'narrativ_list_assets' agent tool action.
//!
//! This function takes `ListAssetsParams`, calls the corresponding query functions,
//! and maps the result to agent-compatible `FullToolResponse` and `UserToolResponse`.
//! It adheres to the standard Rust coding guidelines for this project.

use serde_json::json;

pub async fn handle_list_assets(
    params: crate::agent_tools::tool_params::list_assets_params::ListAssetsParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let user_id = params.user_id.ok_or("The user_id should be provided".to_owned())?;
    let tool_name = "narrativ_list_assets";
    log::debug!("Handling tool call for {tool_name}: {params:?}");

    let limit = params.limit.unwrap_or(20);
    let page = params.page.unwrap_or(0);
    let offset = page * limit;
    let sort_by = params.sort_by.as_deref().unwrap_or("assets.created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("DESC");
    let search_pattern = format!("%{}%", params.search.as_deref().unwrap_or(""));

    // Agent tools only show user's private assets, so empty org_ids is fine
    let org_ids: Vec<uuid::Uuid> = vec![];

    let assets_result = crate::queries::assets::list_assets_with_collection(
        pool,
        user_id,
        &search_pattern,
        sort_by,
        sort_order,
        limit,
        offset,
        None, // collection_id - not supported in agent tools yet
        Some(false), // is_public - agent tools only show user's private assets
        &org_ids,
        None, // logo_related - not filtered in agent tools
    )
    .await;

    // Assumes a count function exists with a similar signature.
    let total_count_result = crate::queries::assets::count_assets(
        pool,
        user_id,
        &search_pattern,
        None, // collection_id - not supported in agent tools yet
        Some(false), // is_public - agent tools only show user's private assets
        &org_ids,
        None, // logo_related - not filtered in agent tools
    )
    .await;

    match (assets_result, total_count_result) {
        (Ok(assets), Ok(total_count)) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: json!({ "assets": assets, "total_count": total_count }),
            };
            let user_response = agentloop::types::user_tool_response::UserToolResponse {
                tool_name: tool_name.to_string(),
                summary: format!("Found {} assets. Displaying {} assets.", total_count.unwrap_or(0), assets.len()),
                data: Some(full_response.response.clone()),
                icon: Some("📦".to_string()),
            };
            Ok((full_response, user_response))
        }
        (Err(e), _) => {
            log::error!("Error fetching assets in {tool_name}: {e:?}");
            Err(format!("Failed to list assets: {e}"))
        }
        (_, Err(e)) => {
            log::error!("Error counting assets in {tool_name}: {e:?}");
            Err(format!("Failed to count assets: {e}"))
        }
    }
}