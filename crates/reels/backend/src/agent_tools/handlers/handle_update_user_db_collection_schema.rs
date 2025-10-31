//! Handles the 'narrativ_update_user_db_collection_schema' agent tool action.
//!
//! This function takes `UpdateUserDbCollectionSchemaParams`, converts its payload,
//! calls the corresponding query function, and maps the result to agent-compatible
//! `FullToolResponse` and `UserToolResponse`. Adheres to Rust coding guidelines.

use serde_json::json;

pub async fn handle_update_user_db_collection_schema(
    params: crate::agent_tools::tool_params::update_user_db_collection_schema_params::UpdateUserDbCollectionSchemaParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let tool_name = "narrativ_update_user_db_collection_schema";

    let query_payload = match params.payload {
        crate::agent_tools::tool_params::update_user_db_collection_schema_params::UpdateUserDbCollectionSchemaPayload::Direct(ref schema_val) => {
            crate::routes::user_db_collections::update_user_db_collection_schema_request::UpdateUserDbCollectionSchemaPayload::Direct {
                schema_definition: schema_val.clone(),
            }
        }
        crate::agent_tools::tool_params::update_user_db_collection_schema_params::UpdateUserDbCollectionSchemaPayload::Instruction(ref instr_val) => {
            crate::routes::user_db_collections::update_user_db_collection_schema_request::UpdateUserDbCollectionSchemaPayload::InstructionBased {
                instruction: instr_val.to_string(),
            }
        }
    };

    match crate::queries::user_db_collections::update_user_db_collection_schema_query::update_user_db_collection_schema_query(
        pool,
        params.user_id.ok_or(String::from("user_id is null"))?,
        params.collection_id,
        query_payload,
    )
    .await
    {
        Ok(collection) => {
            let mut properties = serde_json::Map::new();
            properties.insert(
                "collection".to_string(),
                serde_json::to_value(&collection).unwrap_or(serde_json::Value::Null),
            );

            let full_response = agentloop::types::full_tool_response::FullToolResponse {
                tool_name: tool_name.to_string(),
                response: json!{{"collection": collection}}
            };
          let user_response = agentloop::types::user_tool_response::UserToolResponse {
              tool_name: tool_name.to_string(),
              summary: format!("Successfully updated schema for collection '{}'.", collection.name),
              data: Some(json!{{"collection": collection}}),
               icon: Some("🧱".to_string()),
          };
          Ok((full_response, user_response))
      }
        Err(e) => {
            log::error!("Error in {tool_name}: {e:?}");
            Err(format!("Failed to update user DB collection schema: {e}"))
        }
    }
}
