//! Handles the 'narrativ_create_collection' agent tool action.
//!
//! This function takes `CreateCollectionParams`, calls the corresponding
//! query function, and maps the result to AgentLoop's `FullToolResponse`
//! and `UserToolResponse` structures.

use serde_json::json;

pub async fn handle_create_collection(
    params: crate::agent_tools::tool_params::create_collection_params::CreateCollectionParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let user_id = params.user_id
        .ok_or_else(|| "user_id not found in parameters; this is an internal error.".to_string())?;
    let tool_name = "narrativ_create_collection";
    log::debug!("Handling tool call for {}: {:?}", tool_name, &params);

    let name = params.name.clone();
    let metadata = &params.metadata;
    let organization_id = &params.organization_id;

    let creation_result = crate::queries::collections::create_collection::create_collection(
        pool,
        user_id,
        &name,
        metadata,
        organization_id,
    )
    .await;

    match creation_result {
        Ok(collection) => {
            let response_val = json!(collection.clone());
            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: response_val.clone(),
            };
            let user_response = agentloop::types::user_tool_response::UserToolResponse {
                tool_name: tool_name.to_string(),
                summary: format!("Created collection '{}'", collection.name),
                data: Some(response_val),
                icon: Some("📚".to_string()),
            };
            Ok((full_response, user_response))
        }
        Err(e) => {
            log::error!("Error creating collection in {tool_name}: {e:?}");
            Err(format!("Failed to create collection: {e}"))
        }
    }
}