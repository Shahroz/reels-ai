//! Handles the 'narrativ_get_user_db_collection' agent tool action.
//!
//! This function takes `GetUserDbCollectionParams`, calls the corresponding query function,
//! and maps the result to agent-compatible `FullToolResponse` and `UserToolResponse`.
//! It adheres to the standard Rust coding guidelines for this project.

pub async fn handle_get_user_db_collection(
    params: crate::agent_tools::tool_params::get_user_db_collection_params::GetUserDbCollectionParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_get_user_db_collection";

    match crate::queries::user_db_collections::get_user_db_collection_query::get_user_db_collection_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.collection_id_to_fetch,
    )
    .await
    {
        Ok(Some(collection)) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: serde_json::json!({ "collection": collection }),
            };
            let user_response = agentloop::types::user_tool_response::UserToolResponse {
               tool_name: tool_name.to_string(),
               summary: format!("Successfully fetched collection '{}'.", collection.name),
               data: None,
               icon: Some("📥".to_string()),
           };
           Ok((full_response, user_response))
       }
        Ok(None) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: serde_json::json!({ "found": false, "message": "Collection not found or not accessible." }),
            };
           let user_response = agentloop::types::user_tool_response::UserToolResponse {
               tool_name: tool_name.to_string(),
               summary: format!("Collection (ID: {}) not found or not accessible.", params.collection_id_to_fetch),
               data: None,
               icon: Some("📥".to_string()),
           };
           Ok((full_response, user_response))
       }
        Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to get user DB collection: {e}"))
        }
    }
}
