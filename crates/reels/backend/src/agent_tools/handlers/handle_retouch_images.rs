/// Handles the RetouchImages GN workflow tool execution.
///
/// Creates a workflow using the RetouchImages template with the provided image
/// URIs and optional retouch prompt, then polls for one result item from the workflow execution.
///
/// # Arguments
///
/// * `params` - `crate::agent_tools::tool_params::retouch_images_params::RetouchImagesParams` containing image details.
///
/// # Returns
///
/// A `Result` containing a tuple of
/// `(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse)`
/// on success, or an error `String` on failure.
pub async fn handle_retouch_images(
    params: crate::agent_tools::tool_params::retouch_images_params::RetouchImagesParams,
) -> std::result::Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    std::string::String,
> {
    let tool_name = "retouch_images".to_string();
    
    // Check credit availability before proceeding
    let user_id = params.user_id.ok_or("User id should be provided".to_string())?;
    let organization_id = params.organization_id;
    let credits_to_consume = (params.photos.len() as i32) * crate::app_constants::credits_constants::CreditsConsumption::RETOUCH_IMAGES;
    let pool: &sqlx::PgPool = crate::db_pool::GLOBAL_POOL.get_ref();
    
    if let Err(error) = crate::services::credits_service::check_credits_availability_by_user_or_organization(
        pool,
        user_id,
        credits_to_consume,
        organization_id,
    ).await {
        return std::result::Result::Err(error.message);
    }
    
    // Create gennodes-client instance with timeout (10 min for image processing), retries (3x), and configurable authentication
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
    
    // Create RetouchImages workflow using the specific method
    // Convert our params to the gennodes-client RetouchImagesParams struct
    let gennodes_params = gennodes_client::templates::retouch_images::RetouchImagesParams {
        photos: params.photos.clone(),
        retouch_prompt: params.retouch_prompt.clone(),
        retouch_settings: None
    };
    
    let workflow_id = match client
        .create_retouch_images_workflow(gennodes_params)
        .await
    {
        std::result::Result::Ok(id) => id,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!(
                "Failed to create RetouchImages workflow: {e}"
            ));
        }
    };
    
    // Request items from the workflow using the correct constructor and builder pattern
    // If multiple photos are provided, request up to that many results
    let pull_params = gennodes_client::client::workflow_pull_params::WorkflowPullParams::new(workflow_id, 1)
        .with_simple_output(true);
    
    let results = match client.run_workflow_pull(pull_params).await {
        std::result::Result::Ok(items) => {
            if items.is_empty() {
                return std::result::Result::Err(
                    "No results returned from RetouchImages workflow".to_string(),
                );
            }
            items.clone()
        }
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!(
                "Failed to retrieve results from RetouchImages workflow: {e}"
            ));
        }
    };
    
    let full_response = agentloop::types::full_tool_response::FullToolResponse {
        tool_name: tool_name.clone(),
        response: serde_json::Value::Array(results.clone()),
    };
    
    let user_response = agentloop::types::user_tool_response::UserToolResponse {
        tool_name,
        summary: format!(
            "Retouched {} image(s){}",
            params.photos.len(),
            params.retouch_prompt
                .as_ref()
                .map(|prompt| format!(" with prompt: {prompt}"))
                .unwrap_or_else(|| " using default settings".to_string())
        ),
        data: std::option::Option::Some(serde_json::Value::Array(results)),
        icon: std::option::Option::Some("🎨".to_string()),
    };
    
    // Use provided credit_changes_params or construct from defaults
    let deduction_params = params.credit_changes_params.unwrap_or_else(|| {
        crate::queries::user_credit_allocation::CreditChangesParams {
            user_id,
            organization_id,
            credits_to_change: bigdecimal::BigDecimal::from(credits_to_consume),
            action_source: "agent_tool".to_string(),
            action_type: "retouch_images".to_string(),
            entity_id: None, // No specific entity for retouch
        }
    });
    
    if let Err(e) = crate::queries::user_credit_allocation::deduct_user_credits_with_transaction(pool, deduction_params).await {
        log::error!("Failed to deduct credits for user {} after retouch images: {}", user_id, e);
    }
    
    std::result::Result::Ok((full_response, user_response))
} 