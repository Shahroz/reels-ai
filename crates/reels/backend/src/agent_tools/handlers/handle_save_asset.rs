//! Handles the 'narrativ_save_asset' agent tool action.
//!
//! This function takes `SaveAssetParams` with an array of existing GCS URIs,
//! saves all asset metadata to the database, and returns agent-compatible
//! `FullToolResponse` and `UserToolResponse`. It skips the upload step
//! since the assets already exist in GCS.

use serde_json::json;

pub async fn handle_save_asset(
    params: crate::agent_tools::tool_params::save_asset_params::SaveAssetParams,
    pool: &sqlx::PgPool,
) -> Result<(
    agentloop::types::full_tool_response::FullToolResponse,
    agentloop::types::user_tool_response::UserToolResponse,
), String> {
    let user_id = params.user_id.ok_or("The user_id should be provided".to_owned())?;
    let tool_name = "narrativ_save_asset";
    log::debug!("Handling tool call for {tool_name}: {params:?}");

    if params.assets.is_empty() {
        return Err("No assets provided to save".to_string());
    }

    let mut saved_assets = Vec::new();
    let mut errors = Vec::new();

    // Process each asset in the array
    for (index, asset_data) in params.assets.iter().enumerate() {
        // Parse collection_id if provided
        let parsed_collection_id = asset_data.collection_id.as_ref().and_then(|id| {
            uuid::Uuid::parse_str(id).ok()
        });

        // Generate a new asset ID
        let asset_id = uuid::Uuid::new_v4();

        // Insert asset metadata into the database
        // Note: This handler saves assets from URLs without access to file content,
        // so metadata extraction is not possible here
        let result = crate::queries::assets::create_asset::create_asset(
            pool,
            asset_id,
            Some(user_id),
            &asset_data.name,
            &asset_data.r#type,
            &asset_data.gcs_object_name,
            &asset_data.gcs_url,
            parsed_collection_id,
            None, // No metadata available from URL-based saves
            false // is_public - agent tools create private assets
        )
        .await;

        match result {
            Ok(asset) => {
                log::debug!("Successfully saved asset '{}' with ID {}", asset_data.name, asset.id);
                saved_assets.push(asset);
            }
            Err(e) => {
                let error_msg = format!("Failed to save asset '{}' (index {}): {}", asset_data.name, index, e);
                log::error!("Error saving asset in {tool_name}: {error_msg}");
                errors.push(error_msg);
            }
        }
    }

    // Determine response based on results
    let total_attempted = params.assets.len();
    let total_saved = saved_assets.len();
    let total_failed = errors.len();

    // Update credit reward progress for assets upload
    if let Err(e) = crate::queries::credit_rewards::update_user_reward_progress(
        &pool,
        user_id,
        crate::app_constants::credits_constants::CreditRewardActionTypes::UPLOAD_ASSETS,
        total_saved as i32,
    ).await {
        log::warn!("Failed to update credit reward progress for user {}: {}", user_id, e);
    }

    if total_failed == 0 {
        // All assets saved successfully
        let full_response = agentloop::types::full_tool_response::FullToolResponse {
            tool_name: tool_name.to_string(),
            response: json!({ 
                "assets": saved_assets,
                "total_saved": total_saved,
                "total_failed": total_failed
            }),
        };
        let user_response = agentloop::types::user_tool_response::UserToolResponse {
            tool_name: tool_name.to_string(),
            summary: format!("Successfully saved {} asset{}", total_saved, if total_saved == 1 { "" } else { "s" }),
            data: Some(full_response.response.clone()),
            icon: Some("💾".to_string()),
        };
        Ok((full_response, user_response))
    } else if total_saved == 0 {
        // All assets failed
        Err(format!("Failed to save all {} assets. Errors: {}", total_attempted, errors.join("; ")))
    } else {
        // Partial success - some saved, some failed
        let full_response = agentloop::types::full_tool_response::FullToolResponse {
            tool_name: tool_name.to_string(),
            response: json!({ 
                "assets": saved_assets,
                "total_saved": total_saved,
                "total_failed": total_failed,
                "errors": errors
            }),
        };
        let user_response = agentloop::types::user_tool_response::UserToolResponse {
            tool_name: tool_name.to_string(),
            summary: format!("Saved {total_saved} of {total_attempted} assets. {total_failed} failed."),
            data: Some(full_response.response.clone()),
            icon: Some("⚠️".to_string()),
        };
        Ok((full_response, user_response))
    }
} 