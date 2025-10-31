//! Handles the 'narrativ_update_user_db_collection' agent tool action.
//!
//! This function takes `UpdateUserDbCollectionParams`, calls the corresponding query function,
//! and maps the result to agent-compatible `FullToolResponse` and `UserToolResponse`.
//! It adheres to the standard Rust coding guidelines for this project.

pub async fn handle_update_user_db_collection(
    params: crate::agent_tools::tool_params::update_user_db_collection_params::UpdateUserDbCollectionParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_update_user_db_collection";

    match crate::queries::user_db_collections::update_user_db_collection_query::update_user_db_collection_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.collection_id_to_update,
        params.new_name.clone(),
        params.new_description.clone(),
    )
   .await
   {
       Ok(collection) => {
            // Capture collection.name for summary before collection might be moved/consumed.
            let summary_text = format!("Successfully updated collection '{}'.", collection.name);
            // Create the JSON value from collection. This consumes `collection`.
            let collection_json_data = serde_json::json!({ "collection": collection });

            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: collection_json_data.clone(), // Clone for full_response
            };
           let user_response = agentloop::types::user_tool_response::UserToolResponse {
               tool_name: tool_name.to_string(),
               summary: summary_text,
               data: None,
               icon: Some("🔄".to_string()),
           };
          Ok((full_response, user_response))
      }
       Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to update user DB collection: {e}"))
        }
    }
}
