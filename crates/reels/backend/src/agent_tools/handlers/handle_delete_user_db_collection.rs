//! Handles the 'narrativ_delete_user_db_collection' agent tool action.
//!
//! This function takes `DeleteUserDbCollectionParams`, calls the corresponding query function,
//! and maps the result to agent-compatible `FullToolResponse` and `UserToolResponse`.
//! It adheres to the standard Rust coding guidelines for this project.

pub async fn handle_delete_user_db_collection(
    params: crate::agent_tools::tool_params::delete_user_db_collection_params::DeleteUserDbCollectionParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_delete_user_db_collection";

    match crate::queries::user_db_collections::delete_user_db_collection_query::delete_user_db_collection_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.collection_id_to_delete,
    )
   .await
   {
       Ok(rows_affected) => {
            let summary = if rows_affected > 0 {
                format!("Successfully deleted collection (ID: {}).", params.collection_id_to_delete)
            } else {
                format!("Collection (ID: {}) not found or not owned by user.", params.collection_id_to_delete)
            };

            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: serde_json::json!({ "rows_affected": rows_affected }),
            };
          let user_response = agentloop::types::user_tool_response::UserToolResponse {
              tool_name: tool_name.to_string(),
              summary,
              data: Some(serde_json::json!({ "rows_affected": rows_affected })),
               icon: Some("🗑️".to_string()),
          };
          Ok((full_response, user_response))
      }
        Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to delete user DB collection: {e}"))
        }
    }
}
