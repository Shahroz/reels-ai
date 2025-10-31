//! Handles the 'narrativ_create_user_db_collection_item' agent tool action.
//!
//! This function takes `CreateUserDbCollectionItemParams`, calls the query function,
//! and maps results to `FullToolResponse` and `UserToolResponse`.
//! Adheres to Rust coding guidelines.

pub async fn handle_create_user_db_collection_item(
    params: crate::agent_tools::tool_params::create_user_db_collection_item_params::CreateUserDbCollectionItemParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_create_user_db_collection_item";

    match crate::queries::user_db_collections::items::create_user_db_collection_item_query::create_user_db_collection_item_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.collection_id_uuid,
        params.item_data.clone(),
    )
   .await
   {
       Ok(item) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: serde_json::json!({ "item": item }),
            };
           let user_response = agentloop::types::user_tool_response::UserToolResponse {
               tool_name: tool_name.to_string(),
               summary: format!("Successfully created item (ID: {}) in collection (ID: {}).", item.id, params.collection_id_uuid),
               data: Some(serde_json::json!(item.clone())),
               icon: Some("➕".to_string()),
           };
           Ok((full_response, user_response))
       }
        Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to create user DB collection item: {e}"))
        }
    }
}
