//! Handles the 'narrativ_list_user_db_collections' agent tool action.
//!
//! This function takes `ListUserDbCollectionsParams`, calls the corresponding query function,
//! and maps the result to agent-compatible `FullToolResponse` and `UserToolResponse`.
//! It adheres to the standard Rust coding guidelines for this project.

use serde_json::json;

pub async fn handle_list_user_db_collections(
    params: crate::agent_tools::tool_params::list_user_db_collections_params::ListUserDbCollectionsParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_list_user_db_collections";
    log::error!("{params:?}");

    match crate::queries::user_db_collections::list_user_db_collections_query::list_user_db_collections_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.limit,
        params.offset,
        &params.sort_by_db_col_name,
        &params.sort_order_db,
        params.search_pattern_db.as_deref(),
    )
    .await
    {
        Ok((collections, total_count)) => {
            let mut properties = serde_json::Map::new();
            properties.insert(
                "collections".to_string(),
                serde_json::to_value(&collections).unwrap_or(serde_json::Value::Null),
            );
            properties.insert("total_count".to_string(), serde_json::json!(total_count));

            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: json!{{"collections": collections, "total_count": total_count}},
            };
          let user_response = agentloop::types::user_tool_response::UserToolResponse {
              tool_name: tool_name.to_string(),
              summary: format!("Found {} collections. Displaying {} collections.", total_count, collections.len()),
              data: Some(full_response.response.clone()),
               icon: Some("📚".to_string()),
          };
          Ok((full_response, user_response))
      }
        Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to list user DB collections: {e}"))
        }
    }
}
