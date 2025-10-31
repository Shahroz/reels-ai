//! Handles the 'narrativ_query_user_db_collection_items' agent tool action.
//!
//! This function takes `QueryUserDbCollectionItemsParams`, calls the query function,
//! and maps results to `FullToolResponse` and `UserToolResponse`.
//! Adheres to Rust coding guidelines.

pub async fn handle_query_user_db_collection_items(
    params: crate::agent_tools::tool_params::query_user_db_collection_items_params::QueryUserDbCollectionItemsParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_query_user_db_collection_items";

    match crate::queries::user_db_collections::items::query_user_db_collection_items_query::query_user_db_collection_items_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.collection_id,
        &params.query_string,
        params.page,
        params.limit,
    )
   .await
   {
       Ok((items, total_count)) => {
            let num_items_returned = items.len(); // Store len before items might be moved
            let response_payload = serde_json::json!({ "items": items, "total_count": total_count }); // items moved here

            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: response_payload.clone(), // Clone the payload
            };
            let user_response = agentloop::types::user_tool_response::UserToolResponse {
                tool_name: tool_name.to_string(),
                summary: format!("Query returned {} items from collection (ID: {}). Displaying {} items.", total_count, params.collection_id, num_items_returned),
                icon: Some("🔍".to_string()),
                data: Some(response_payload), // Use the original payload
            };
           Ok((full_response, user_response))
       }
       Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to query user DB collection items: {e}"))
        }
    }
}
