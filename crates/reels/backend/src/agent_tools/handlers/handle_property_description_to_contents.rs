//! Implements the logic for the PropertyDescriptionToContents GN workflow tool.
//!
//! This function takes property description and generates comprehensive marketing content
//! collection using the specialized gennodes-client method. The workflow converts property
//! descriptions into various types of marketing content suitable for real estate purposes.
//! Uses the type-safe create_property_desc_to_contents_workflow method for better parameter validation.
//! Configuration via required environment variables:
//! - GENNODES_BASE_URL: Base URL for gennodes server (required)
//! - GENNODES_USERNAME: Username for basic auth (required)
//! - GENNODES_PASSWORD: Password for basic auth (required)
//! 
//! Client configuration:
//! - Timeout: 10 minutes (to handle content generation workload)
//! - Retries: 3 attempts with 100ms exponential backoff
//! Adheres to Narrativ and AgentLoop coding standards with one item per file structure.

/// Handles the PropertyDescriptionToContents GN workflow tool execution.
///
/// Creates a workflow using the PropertyDescriptionToContents template with the provided property
/// description, then polls for one result item from the workflow execution.
///
/// # Arguments
///
/// * `params` - `crate::agent_tools::tool_params::property_description_to_contents_params::PropertyDescriptionToContentsParams` containing property description.
///
/// # Returns
///
/// A `Result` containing a tuple of
/// `(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse)`
/// on success, or an error `String` on failure.
pub async fn handle_property_description_to_contents(
    params: crate::agent_tools::tool_params::property_description_to_contents_params::PropertyDescriptionToContentsParams,
) -> std::result::Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    std::string::String,
> {
    let tool_name = "property_description_to_contents".to_string();
    
    // Create gennodes-client instance with timeout (10 min for content generation), retries (3x), and configurable authentication
    let base_url = std::env::var("GENNODES_BASE_URL")
        .map_err(|_| "GENNODES_BASE_URL environment variable is required".to_string())?;
    let username = std::env::var("GENNODES_USERNAME")
        .map_err(|_| "GENNODES_USERNAME environment variable is required".to_string())?;
    let password = std::env::var("GENNODES_PASSWORD")
        .map_err(|_| "GENNODES_PASSWORD environment variable is required".to_string())?;

    let client = match gennodes_client::client::core::Client::new_with_timeout(&base_url, std::time::Duration::from_secs(600)) {  // 10 minutes timeout
        std::result::Result::Ok(c) => c
            .with_retries(3, std::option::Option::Some(100)) // 3 retries with 100ms exponential backoff
            .with_auth(gennodes_client::auth::AuthConfig::basic(&username, &password)),
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!(
                "Failed to create gennodes client: {e}"
            ));
        }
    };
    
    // Create PropertyDescriptionToContents workflow using the specific method
    // Convert our params to the gennodes-client PropertyDescToContentsParams struct
    let gennodes_params = gennodes_client::templates::property_desc_to_contents::PropertyDescToContentsParams::new(
        params.property_info.clone()
    );
    
    let workflow_id = match client
        .create_property_desc_to_contents_workflow(gennodes_params)
        .await
    {
        std::result::Result::Ok(id) => id,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!(
                "Failed to create PropertyDescriptionToContents workflow: {e}"
            ));
        }
    };
    
    // Request one item from the workflow using the correct constructor and builder pattern
    let pull_params = gennodes_client::client::workflow_pull_params::WorkflowPullParams::new(workflow_id, 1)
        .with_simple_output(true);
    
    let result = match client.run_workflow_pull(pull_params).await {
        std::result::Result::Ok(items) => {
            if items.is_empty() {
                return std::result::Result::Err(
                    "No results returned from PropertyDescriptionToContents workflow".to_string(),
                );
            }
            items[0].clone()
        }
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!(
                "Failed to retrieve results from PropertyDescriptionToContents workflow: {e}"
            ));
        }
    };
    
    let full_response = agentloop::types::full_tool_response::FullToolResponse {
        tool_name: tool_name.clone(),
        response: result.clone(),
    };
    
    let user_response = agentloop::types::user_tool_response::UserToolResponse {
        tool_name,
        summary: format!(
            "Generated marketing content collection from property description"
        ),
        data: std::option::Option::Some(result),
        icon: std::option::Option::Some("📄".to_string()),
    };
    
    std::result::Result::Ok((full_response, user_response))
} 