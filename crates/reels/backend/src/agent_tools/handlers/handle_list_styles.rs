//! Handles the 'narrativ_list_styles' agent tool action.
//!
//! This function takes `ListStylesParams`, calls the corresponding query functions,
//! and maps the result to agent-compatible `FullToolResponse` and `UserToolResponse`.
//! It adheres to the standard Rust coding guidelines for this project.

use serde_json::json;

pub async fn handle_list_styles(
    params: crate::agent_tools::tool_params::list_styles_params::ListStylesParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let user_id = params.user_id.ok_or("The user_id should be provided".to_owned())?;
    let tool_name = "narrativ_list_styles";
    log::debug!("Handling tool call for {tool_name}: {params:?}");

    // The 'is_favorite' parameter is part of ListStylesParams but not supported
    // by the underlying query functions. Log a warning if it's used.
    if params.is_favorite.is_some() {
        log::warn!(
            "The 'is_favorite' filter is not yet implemented for the '{tool_name}' tool."
        );
    }

    // Pagination and sorting parameters
    let limit = params.limit.unwrap_or(20);
    let page = params.page.unwrap_or(0);
    let offset = page * limit;
    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("DESC");
    let is_favourite = params.is_favorite;
    let search_pattern = format!("%{}%", params.search.as_deref().unwrap_or(""));

    // Fetch organization IDs for the user, as required by the queries.
    let org_memberships =
        match crate::queries::organizations::find_active_memberships_for_user::find_active_memberships_for_user(pool, user_id).await
        {
            Ok(memberships) => memberships,
            Err(e) => {
                let err_msg = format!("Failed to retrieve user organization memberships: {e}");
                log::error!("{err_msg}");
                return Err(err_msg);
            }
        };
    let org_ids: std::vec::Vec<uuid::Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();

    // Fetch styles
    let styles_result = crate::queries::styles::list_styles_for_user::list_styles_for_user(
        pool,
        user_id,
        &search_pattern,
        &org_ids,
        limit,
        offset,
        sort_by,
        sort_order,
        is_favourite,
        None // is_public_filter: None means return both public and private styles
    )
    .await;

    // Fetch total count
    let total_count_result = crate::queries::styles::count_styles_for_user::count_styles_for_user(
        pool,
        user_id,
        &search_pattern,
        &org_ids,
        is_favourite,
        None // is_public_filter: None means count both public and private styles
    )
    .await;

    match (styles_result, total_count_result) {
        (Ok(styles), Ok(total_count)) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: json!({ "styles": styles, "total_count": total_count }),
            };
            let user_response = agentloop::types::user_tool_response::UserToolResponse {
                tool_name: tool_name.to_string(),
                summary: format!("Found {} styles. Displaying {} styles.", total_count, styles.len()),
                data: Some(full_response.response.clone()),
                icon: Some("🎨".to_string()),
            };
            Ok((full_response, user_response))
        }
        (Err(e), _) => {
            log::error!("Error fetching styles in {tool_name}: {e:?}");
            Err(format!("Failed to list styles: {e}"))
        }
        (_, Err(e)) => {
            log::error!("Error counting styles in {tool_name}: {e:?}");
            Err(format!("Failed to count styles: {e}"))
        }
    }
}