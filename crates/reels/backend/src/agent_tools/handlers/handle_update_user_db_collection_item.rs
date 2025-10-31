
//! Handles the 'narrativ_update_user_db_collection_item' tool action for merging data.
//!
//! This function takes `UpdateUserDbCollectionItemParams`, calls the merge query,
//! and maps the results to `FullToolResponse` and `UserToolResponse`.
//! It now reflects a merge/patch operation instead of a full update.

pub async fn handle_update_user_db_collection_item(
    params: crate::agent_tools::tool_params::update_user_db_collection_item_params::UpdateUserDbCollectionItemParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_update_user_db_collection_item";

    match crate::queries::user_db_collections::items::update_user_db_collection_item_query::update_user_db_collection_item_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.collection_id_uuid,
        params.item_id_uuid,
        params.item_data_patch.clone(),
    )
   .await
   {
       Ok(item) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: serde_json::json!({ "item": item })
            };
          let user_response = agentloop::types::user_tool_response::UserToolResponse {
              tool_name: tool_name.to_string(),
              summary: format!("Successfully merged data into item (ID: {}) in collection (ID: {}).", item.id, params.collection_id_uuid),
              data: Some(serde_json::json!({ "item": item.clone() })),
               icon: Some("🔄".to_string()),
          };
          Ok((full_response, user_response))
      }
        Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to update user DB collection item: {e}"))
        }
    }
}
