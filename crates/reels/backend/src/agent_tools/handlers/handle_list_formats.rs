//! Handles the 'narrativ_list_formats' agent tool action.
//!
//! This function takes `ListFormatsParams`, calls the corresponding query functions,
//! and maps the result to agent-compatible `FullToolResponse` and `UserToolResponse`.
//! It adheres to the standard Rust coding guidelines for this project.

use serde_json::json;

pub async fn handle_list_formats(
    params: crate::agent_tools::tool_params::list_formats_params::ListFormatsParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let user_id = params.user_id.ok_or("The user_id should be provided".to_owned())?;
    let tool_name = "narrativ_list_formats";
    log::debug!("Handling tool call for {tool_name}: {params:?}");

    let limit = params.limit.unwrap_or(20);
    let page = params.page.unwrap_or(0);
    let offset = page * limit;
    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("desc"); 
    let search = params.search.clone();
    let is_public = params.is_public.unwrap_or(false);

    let formats_result = crate::queries::custom_creative_formats::list::list_formats(
        pool,
        user_id,
        is_public,
        search,
        sort_by,
        sort_order,
        limit,
        offset,
    )
    .await;

    // Assumes a count function exists with a similar signature.
    let total_count_result = crate::queries::custom_creative_formats::count::count_formats(
        pool,
        user_id,
        is_public,
        params.search,
    )
    .await;

    match (formats_result, total_count_result) {
        (Ok(formats), Ok(total_count)) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: json!({ "formats": formats, "total_count": total_count }),
            };
            let user_response = agentloop::types::user_tool_response::UserToolResponse {
                tool_name: tool_name.to_string(),
                summary: format!("Found {} formats. Displaying {} formats.", total_count, formats.len()),
                data: Some(full_response.response.clone()),
                icon: Some("📏".to_string()),
            };
            Ok((full_response, user_response))
        }
        (Err(e), _) => {
            log::error!("Error fetching formats in {tool_name}: {e:?}");
            Err(format!("Failed to list formats: {e}"))
        }
        (_, Err(e)) => {
            log::error!("Error counting formats in {tool_name}: {e:?}");
            Err(format!("Failed to count formats: {e}"))
        }
    }
}