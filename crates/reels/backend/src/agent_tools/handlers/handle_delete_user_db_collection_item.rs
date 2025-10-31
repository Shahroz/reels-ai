//! Handles the 'narrativ_delete_user_db_collection_item' agent tool action.
//!
//! This function takes `DeleteUserDbCollectionItemParams`, calls the query function,
//! and maps results to `FullToolResponse` and `UserToolResponse`.
//! Adheres to Rust coding guidelines.

use serde_json::json;

pub async fn handle_delete_user_db_collection_item(
    params: crate::agent_tools::tool_params::delete_user_db_collection_item_params::DeleteUserDbCollectionItemParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_delete_user_db_collection_item";

    match crate::queries::user_db_collections::items::delete_user_db_collection_item_query::delete_user_db_collection_item_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.collection_id_uuid,
        params.item_id_uuid,
    )
    .await
    {
        Ok(rows_affected) => {
            let mut properties = serde_json::Map::new();
            properties.insert("rows_affected".to_string(), serde_json::json!(rows_affected));

            let summary = if rows_affected > 0 {
                format!("Successfully deleted item (ID: {}) from collection (ID: {}).", params.item_id_uuid, params.collection_id_uuid)
            } else {
                format!("Item (ID: {}) not found in collection (ID: {}) or not accessible.", params.item_id_uuid, params.collection_id_uuid)
            };

            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: json!{{"rows_affected": rows_affected}},
            };
          let user_response = agentloop::types::user_tool_response::UserToolResponse {
              tool_name: tool_name.to_string(),
              summary,
               data: None,
               icon: Some("🗑️".to_string()),
          };
          Ok((full_response, user_response))
      }
        Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to delete user DB collection item: {e}"))
        }
    }
}
