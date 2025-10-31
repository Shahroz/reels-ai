//! Implements the logic for the PropertyResearch GN workflow tool.
//!
//! This function takes property identifier and optional file URIs, creates a PropertyResearch
//! workflow using the specialized gennodes-client method, and retrieves one result item.
//! The workflow researches property information using web search, AI analysis of provided files,
//! and Perplexity to gather information and populate an MLS Entry form.
//! Uses the type-safe create_property_research_workflow method for better parameter validation.
//! Configuration via required environment variables:
//! - GENNODES_BASE_URL: Base URL for gennodes server (required)
//! - GENNODES_USERNAME: Username for basic auth (required)
//! - GENNODES_PASSWORD: Password for basic auth (required)
//! 
//! Client configuration:
//! - Timeout: 5 minutes (default from gennodes-client)
//! - Retries: 3 attempts with 100ms exponential backoff
//! Adheres to Narrativ and AgentLoop coding standards with one item per file structure.

use crate::queries::user_credit_allocation::{deduct_user_credits_with_transaction, CreditChangesParams};
use bigdecimal::BigDecimal;

/// Handles the PropertyResearch GN workflow tool execution.
///
/// Creates a workflow using the PropertyResearch template with the provided property
/// identifier and optional file URIs, then polls for one result item from the workflow execution.
///
/// # Arguments
///
/// * `params` - `crate::agent_tools::tool_params::property_research_params::PropertyResearchParams` containing property details.
///
/// # Returns
///
/// A `Result` containing a tuple of
/// `(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse)`
/// on success, or an error `String` on failure.
pub async fn handle_property_research(
    pool: &sqlx::PgPool,
    params: crate::agent_tools::tool_params::property_research_params::PropertyResearchParams,
    user_id: uuid::Uuid,
) -> std::result::Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    std::string::String,
> {
    let tool_name = "property_research".to_string();
    let organization_id = params.organization_id;
    
    // Check credit availability before proceeding
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::PROPERTY_RESEARCH;
    
    if let Err(error) = crate::services::credits_service::check_credits_availability_by_user_or_organization(
        pool,
        user_id,
        credits_to_consume,
        organization_id,
    ).await {
        return std::result::Result::Err(error.message);
    }
    
    // Create gennodes-client instance with timeout (5 min default), retries (3x), and configurable authentication
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
    
    // Create PropertyResearch workflow using the specific method
    // Convert our params to the gennodes-client PropertyResearchParams struct
    let gennodes_params = gennodes_client::templates::property_research::PropertyResearchParams {
        property_identifier: params.property_identifier.clone(),
        documents: params.documents.clone(),
        photos: params.photos.clone(),
        videos: params.videos.clone(),
        text_documents: None,
    };
    
    let workflow_id = match client
        .create_property_research_workflow(gennodes_params)
        .await
    {
        std::result::Result::Ok(id) => id,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!(
                "Failed to create PropertyResearch workflow: {e}"
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
                    "No results returned from PropertyResearch workflow".to_string(),
                );
            }
            items[0].clone()
        }
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!(
                "Failed to retrieve results from PropertyResearch workflow: {e}"
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
            "Generated property research for {}",
            params.property_identifier
        ),
        data: std::option::Option::Some(result),
        icon: std::option::Option::Some("🏠".to_string()),
    };
    
    // Consume credits before returning response
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::PROPERTY_RESEARCH;
    let deduction_params = CreditChangesParams {
        user_id,
        organization_id, // Use organization_id from params
        credits_to_change: BigDecimal::from(credits_to_consume),
        action_source: "agent_tool".to_string(),
        action_type: "property_research".to_string(),
        entity_id: None, // No specific entity for research
    };
    if let Err(e) = deduct_user_credits_with_transaction(pool, deduction_params).await {
        log::error!("Failed to deduct {} credits for user {} after property research: {}", credits_to_consume, user_id, e);
    }

    std::result::Result::Ok((full_response, user_response))
}
