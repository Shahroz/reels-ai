//! Handles the 'narrativ_list_user_db_collection_items' agent tool action.
//!
//! This function takes `ListUserDbCollectionItemsToolParams`, calls the query function,
//! and maps results to `FullToolResponse` and `UserToolResponse`.
//! Adheres to Rust coding guidelines.

pub async fn handle_list_user_db_collection_items_tool(
    params: crate::agent_tools::tool_params::list_user_db_collection_items_tool_params::ListUserDbCollectionItemsToolParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_list_user_db_collection_items";

    match crate::queries::user_db_collections::items::list_user_db_collection_items_query::list_user_db_collection_items_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.collection_id_uuid,
        params.page,
        params.limit,
        &params.sort_by_column_name,
        &params.sort_order,
        params.search_pattern.as_deref(),
    )
   .await
   {
       Ok((items, total_count)) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
               response: serde_json::json!({ "items": items, "total_count": total_count })
           };
          let user_response = agentloop::types::user_tool_response::UserToolResponse {
              tool_name: tool_name.to_string(),
              summary: format!("Found {} items in collection (ID: {}). Displaying {} items.", total_count, params.collection_id_uuid, items.len()),
              data: Some(serde_json::json!({ "items": items, "total_count": total_count })),
               icon: Some("📜".to_string()),
          };
          Ok((full_response, user_response))
      }
        Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to list user DB collection items: {e}"))
        }
    }
}
