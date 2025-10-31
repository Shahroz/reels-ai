//! Handles the 'narrativ_get_user_db_collection_item' agent tool action.
//!
//! This function takes `GetUserDbCollectionItemParams`, calls the query function,
//! and maps results to `FullToolResponse` and `UserToolResponse`.
//! Adheres to Rust coding guidelines.

pub async fn handle_get_user_db_collection_item(
    params: crate::agent_tools::tool_params::get_user_db_collection_item_params::GetUserDbCollectionItemParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_get_user_db_collection_item";

    match crate::queries::user_db_collections::items::get_user_db_collection_item_query::get_user_db_collection_item_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.collection_id_uuid,
        params.item_id_uuid,
    )
   .await
   {
       Ok(Some(item)) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: serde_json::json!({ "item": item })
            };
          let user_response = agentloop::types::user_tool_response::UserToolResponse {
              tool_name: tool_name.to_string(),
              summary: format!("Successfully fetched item (ID: {}) from collection (ID: {}).", item.id, params.collection_id_uuid),
              data: Some(serde_json::json!({ "item_id": item.id, "collection_id": params.collection_id_uuid, "success": true })),
               icon: Some("📄".to_string()),
          };
          Ok((full_response, user_response))
      }
        Ok(None) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: serde_json::json!({ "found": false, "message": "Item not found or not accessible." })
            };
          let user_response = agentloop::types::user_tool_response::UserToolResponse {
              tool_name: tool_name.to_string(),
              summary: format!("Item (ID: {}) not found in collection (ID: {}) or not accessible.", params.item_id_uuid, params.collection_id_uuid),
              data: Some(serde_json::json!({ "item_id": params.item_id_uuid, "collection_id": params.collection_id_uuid, "found": false })),
               icon: Some("📄".to_string()),
          };
          Ok((full_response, user_response))
      }
        Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to get user DB collection item: {e}"))
        }
    }
}
