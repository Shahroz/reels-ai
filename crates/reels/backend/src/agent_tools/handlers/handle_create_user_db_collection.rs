//! Handles the 'narrativ_create_user_db_collection' agent tool action.
//!
//! This function takes `CreateUserDbCollectionParams`, calls the corresponding query function,
//! and maps the result to agent-compatible `FullToolResponse` and `UserToolResponse`.
//! It adheres to the standard Rust coding guidelines for this project.

pub async fn handle_create_user_db_collection(
    params: crate::agent_tools::tool_params::create_user_db_collection_params::CreateUserDbCollectionParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_create_user_db_collection";

    match crate::queries::user_db_collections::create_user_db_collection_query::create_user_db_collection_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.name.clone(), // Clone if String is consumed by query
        params.description.clone(), // Clone if Option<String> is consumed
        params.initial_schema_definition.clone(), // Clone if Value is consumed
    )
   .await
   {
       Ok(collection) => {
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: serde_json::json!({ "collection": collection }),
            };
           let user_response = agentloop::types::user_tool_response::UserToolResponse {
               tool_name: tool_name.to_string(),
               summary: format!("Successfully created collection '{}'.", collection.name),
               data: Some(serde_json::json!({ "collection": collection })),
               icon: Some("✨".to_string()),
           };
           Ok((full_response, user_response))
       }
        Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to create user DB collection: {e}"))
        }
    }
}
