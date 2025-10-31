//! Implements the logic for the VocalTour GN workflow tool.
//!
//! This function takes optional file URIs, creates a VocalTour workflow using the 
//! specialized gennodes-client method, and retrieves one result item.
//! The workflow researches property information using AI analysis of provided files 
//! (videos, photos, documents) to gather information and populate a Property Description.
//! Uses the type-safe create_vocal_tour_workflow method for better parameter validation.
//! Configuration via required environment variables:
//! - GENNODES_BASE_URL: Base URL for gennodes server (required)
//! - GENNODES_USERNAME: Username for basic auth (required)
//! - GENNODES_PASSWORD: Password for basic auth (required)
//! 
//! Client configuration:
//! - Timeout: 10 minutes (600 seconds)
//! - Retries: 3 attempts with 100ms exponential backoff
//! Adheres to Reels and AgentLoop coding standards with one item per file structure.

use tracing::instrument;

/// Handles the VocalTour GN workflow tool execution.
///
/// Creates a workflow using the VocalTour template with the provided optional file URIs,
/// then polls for one result item from the workflow execution.
///
/// # Arguments
///
/// * `params` - `crate::agent_tools::tool_params::vocal_tour_params::VocalTourParams` containing file URIs.
///
/// # Returns
///
/// A `Result` containing a tuple of
/// `(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse)`
/// on success, or an error `String` on failure.
#[instrument(name = "vocal_tour", fields(vocal_tour.success, vocal_tour.error_reason, vocal_tour.asset_count, vocal_tour.image_count))]
pub async fn handle_vocal_tour(
    pool: &sqlx::PgPool,
    params: crate::agent_tools::tool_params::vocal_tour_params::VocalTourParams,
    user_id: uuid::Uuid,
) -> std::result::Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    std::string::String,
> {
    let tool_name = "vocal_tour".to_string();
    
    // Check credit availability before proceeding
    /*let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::VOCAL_TOUR;
    match crate::db::user_credit_allocation::check_credits_availability(pool, user_id, credits_to_consume).await {
        Ok(has_credits) => {
            if !has_credits {
                return std::result::Result::Err(format!(
                    "Insufficient credits. This operation requires {} credits but you don't have enough credits remaining.",
                    credits_to_consume
                ));
            }
        }
        Err(e) => {
            log::error!("Failed to check credit availability for user {}: {}", user_id, e);
            return std::result::Result::Err(format!(
                "Failed to verify credit availability. Please try again later."
            ));
        }
    }*/
    
    // Create gennodes-client instance with timeout (10 min), retries (3x), and configurable authentication
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
    
    // Validate that at least one file type is provided
    let has_files = params.documents.as_ref().is_some_and(|d| !d.is_empty()) ||
                   params.photos.as_ref().is_some_and(|p| !p.is_empty()) ||
                   params.videos.as_ref().is_some_and(|v| !v.is_empty());
    
    if !has_files {
        return std::result::Result::Err(
            "VocalTour requires at least one file (document, photo, or video) to analyze".to_string(),
        );
    }
    
    // Create VocalTour workflow using the specific method
    // Convert our params to the gennodes-client VocalTourParams struct
    let gennodes_params = gennodes_client::templates::vocal_tour::VocalTourParams {
        documents: params.documents.clone(),
        photos: params.photos.clone(),
        retouch_prompt: params.retouch_prompt.clone(),
        retouch_settings: None,
        videos: params.videos.clone(),
    };
    
    let workflow_id = match client
        .create_vocal_tour_workflow(gennodes_params)
        .await
    {
        std::result::Result::Ok(id) => id,
        std::result::Result::Err(e) => {
            // Add span attributes for workflow creation failure
            tracing::Span::current().record("vocal_tour.success", false);
            tracing::Span::current().record("vocal_tour.error_reason", "workflow_creation_failed");
            tracing::Span::current().record("vocal_tour.asset_count", 0_u32);
            
            return std::result::Result::Err(format!(
                "Failed to create VocalTour workflow: {e}"
            ));
        }
    };
    
    // Request one item from the workflow using the correct constructor and builder pattern
    let pull_params = gennodes_client::client::workflow_pull_params::WorkflowPullParams::new(workflow_id, 1)
        .with_simple_output(true);
    
    let result = match client.run_workflow_pull(pull_params).await {
        std::result::Result::Ok(items) => {
            if items.is_empty() {
                // Add span attributes for failed vocal-tour
                tracing::Span::current().record("vocal_tour.success", false);
                tracing::Span::current().record("vocal_tour.error_reason", "no_results");
                tracing::Span::current().record("vocal_tour.asset_count", 0_u32);
                
                return std::result::Result::Err(
                    "No results returned from VocalTour workflow".to_string(),
                );
            }
            items[0].clone()
        }
        std::result::Result::Err(e) => {
            // Add span attributes for failed vocal-tour  
            tracing::Span::current().record("vocal_tour.success", false);
            tracing::Span::current().record("vocal_tour.error_reason", "workflow_pull_failed");
            tracing::Span::current().record("vocal_tour.asset_count", 0_u32);
            
            return std::result::Result::Err(format!(
                "Failed to retrieve results from VocalTour workflow: {e}"
            ));
        }
    };
    
    
    // Add comprehensive span attributes for successful vocal-tour
    tracing::Span::current().record("vocal_tour.success", true);
    
    // Add input file counts for correlation analysis
    let input_document_count = params.documents.as_ref().map_or(0, |d| d.len()) as u32;
    let input_photo_count = params.photos.as_ref().map_or(0, |p| p.len()) as u32;
    let input_video_count = params.videos.as_ref().map_or(0, |v| v.len()) as u32;
    
    tracing::Span::current().record("vocal_tour.input_document_count", input_document_count);
    tracing::Span::current().record("vocal_tour.input_photo_count", input_photo_count);
    tracing::Span::current().record("vocal_tour.input_video_count", input_video_count);
    tracing::Span::current().record("vocal_tour.had_retouch_prompt", params.retouch_prompt.is_some());
    
    let full_response = agentloop::types::full_tool_response::FullToolResponse {
        tool_name: tool_name.clone(),
        response: result.clone(),
    };
    
    let user_response = agentloop::types::user_tool_response::UserToolResponse {
        tool_name,
        summary: "Generated property description using AI analysis of provided files.".to_string(),
        data: std::option::Option::Some(result),
        icon: std::option::Option::Some("🎤".to_string()),
    };
    
    // Consume credits before returning response
    /*
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::VOCAL_TOUR;
    if let Err(e) = crate::db::user_credit_allocation::deduct_user_credits_with_transaction(pool, user_id, credits_to_consume).await {
        log::error!("Failed to deduct {} credits for user {} after vocal tour: {}", credits_to_consume, user_id, e);
    }
    */
    std::result::Result::Ok((full_response, user_response))
} 