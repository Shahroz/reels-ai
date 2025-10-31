//! Implements the logic for the Narrativ 'google_search_browse' tool.
//!
//! This function uses Gemini with its grounded search capability to either perform a Google search
//! or browse a direct URL, then process the content based on an extraction query and return a summary.
//! Adheres to Narrativ and AgentLoop coding standards.

use llm::vendors::gemini::gemini_output::GeminiOutput;
use llm::vendors::gemini::tool::URLContext;
use llm::vendors::gemini::tool::URLContextToolWrapper;

use crate::queries::user_credit_allocation::{deduct_user_credits_with_transaction, CreditChangesParams};
use bigdecimal::BigDecimal;

pub async fn handle_google_search_browse(
    params: crate::agent_tools::tool_params::google_search_browse_params::GoogleSearchBrowseParams,
    user_id: uuid::Uuid,
    pool: &sqlx::PgPool,
) -> Result<(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse), String> {
    // Check credit availability before proceeding
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::GOOGLE_SEARCH_BROWSE;
    
    if let Err(error) = crate::services::credits_service::check_credits_availability_by_user_or_organization(
        pool,
        user_id,
        credits_to_consume,
        None, // organization_id not available in params yet
    ).await {
        return std::result::Result::Err(error.message);
    }
    
    // 1. Construct the prompt for Gemini based on provided parameters.
    let prompt = if let Some(url) = &params.url {
        // If a URL is provided, ask Gemini to ground its response in the content of that URL.
        format!(
            "Based on the content of the website at {}, please answer the following query: \"{}\"",
            url, params.extraction_query
        )
    } else if let Some(search_query) = &params.search_query {
        // If a search query is provided, use it along with the extraction query.
        format!("please find informatio about '{}', please focus on extracting '{}'", search_query, params.extraction_query)
    } else {
        // If neither is provided, return an error.
        return Err("Either 'url' or 'search_query' must be provided.".to_string());
    };

    // 2. Configure Gemini to use the Google Search tool for grounding.
    let google_search_tool = llm::vendors::gemini::tool::Tool::GoogleSearch(
        llm::vendors::gemini::tool::GoogleSearchToolWrapper {
            google_search: llm::vendors::gemini::google_search::GoogleSearch {},
        },
    );
    let url_context_tool = llm::vendors::gemini::tool::Tool::URLContext(
        URLContextToolWrapper {
            url_context: URLContext{}
        },
    );
    // Gemini stopped supporting UrlContext
    let tools = Some(vec![google_search_tool]);
    let model = llm::vendors::gemini::gemini_model::GeminiModel::Gemini25Flash;
    let temperature = 0.7; // A moderate temperature for generative but grounded responses.

    // 3. Call Gemini to get the summary.
    let gemini_result = llm::vendors::gemini::completion::generate_gemini_response(
        &prompt,
        temperature,
        model,
        tools,
    )
    .await
    .map_err(|e| {
        let err_msg = format!("Gemini search call failed for prompt '{prompt}': {e}");
        // log::error!("{}", err_msg); // Assuming a logging facade is available.
        err_msg
    })?;

    let summary = match gemini_result {
        llm::vendors::gemini::gemini_output::GeminiOutput::Text(text) => text,
        llm::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(fc) => {
            // This shouldn't happen with grounded search, which should return text.
            let err_msg = format!("Expected text response from Google Search tool, but got a function call: {fc:?}");
            // log::error!("{}", err_msg);
            return Err(err_msg);
        }
        GeminiOutput::Mixed { text, .. } => {
            text
        }
        llm::vendors::gemini::gemini_output::GeminiOutput::Image(_) => {
            let err_msg = "Expected text response from Google Search tool, but got image data".to_string();
            return Err(err_msg);
        }
    };

    // 4. Format the output into the required response structs.
    let tool_name = "google_search_browse".to_string();
    let user_response = agentloop::types::user_tool_response::UserToolResponse {
        tool_name: tool_name.clone(),
        summary: summary.clone(),
        icon: None,
        data: None,
    };

    // Include original params and the final summary in the full response for logging/debugging.
    let full_response_json = serde_json::json!({
        "params": params,
        "final_summary": &summary,
    });

    let full_response = agentloop::types::full_tool_response::FullToolResponse {
        tool_name,
        response: full_response_json,
    };

    // Consume credits before returning response
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::GOOGLE_SEARCH_BROWSE;
    let deduction_params = CreditChangesParams {
        user_id,
        organization_id: None, // TODO: Get from user context if available
        credits_to_change: BigDecimal::from(credits_to_consume),
        action_source: "agent_tool".to_string(),
        action_type: "google_search_browse".to_string(),
        entity_id: None, // No specific entity for search browsing
    };
    if let Err(e) = deduct_user_credits_with_transaction(pool, deduction_params).await {
        log::error!("Failed to deduct {} credits for user {} after google search browse: {}", credits_to_consume, user_id, e);
    }

    Ok((full_response, user_response))
}
