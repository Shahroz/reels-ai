//! Implements the logic for the Narrativ browse_raw tool.
//!
//! This function takes a URL, fetches the raw content, and returns it.
//! Adheres to Narrativ and AgentLoop coding standards.

// Note: Using fully qualified paths as per guidelines where applicable.

use crate::queries::user_credit_allocation::{deduct_user_credits_with_transaction, CreditChangesParams};
use bigdecimal::BigDecimal;

pub async fn handle_narrativ_browse_raw(
    params: crate::agent_tools::tool_params::browse_raw_params::BrowseRawParams,
    user_id: uuid::Uuid,
    pool: &sqlx::PgPool,
) -> Result<(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse), String> {
    let tool_name = "narrativ_browse_raw".to_string();
    
    // Check credit availability before proceeding
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::NARRATIV_BROWSE_RAW;
    
    if let Err(error) = crate::services::credits_service::check_credits_availability_by_user_or_organization(
        pool,
        user_id,
        credits_to_consume,
        None, // organization_id not available in params yet
    ).await {
        return std::result::Result::Err(error.message);
    }
    
    // Assuming api_tools::zyte::fetch_browser_html_and_extract_text is available
    // and returns Result<TextExtraction, String> where TextExtraction has a `content: String` field.
    match api_tools::zyte::fetch_browser_html::fetch_browser_html_and_extract_text(&params.url).await {
        Ok(text_extraction) => {
            let user_response_summary = format!("Browsing (raw) for '{}' completed.", params.url);

            let mut full_response_map = serde_json::Map::new();
            full_response_map.insert("status".to_string(), serde_json::json!("success"));
            full_response_map
                .insert("content".to_string(), serde_json::json!(text_extraction.content));
            
            let user_response_data = Some(serde_json::json!(user_response_summary.clone()));

            let full_response_properties = serde_json::Value::Object(full_response_map);

            // Consume credits before returning response
            let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::NARRATIV_BROWSE_RAW;
            let deduction_params = CreditChangesParams {
                user_id,
                organization_id: None, // TODO: Get from user context if available
                credits_to_change: BigDecimal::from(credits_to_consume),
                action_source: "agent_tool".to_string(),
                action_type: "narrativ_browse_raw".to_string(),
                entity_id: None, // No specific entity for raw browsing
            };
            if let Err(e) = deduct_user_credits_with_transaction(pool, deduction_params).await {
                log::error!("Failed to deduct {} credits for user {} after narrativ browse raw: {}", credits_to_consume, user_id, e);
            }

            Ok((
                agentloop::types::full_tool_response::FullToolResponse {
                    tool_name: tool_name.clone(),
                    response: full_response_properties.clone()
                },
                agentloop::types::user_tool_response::UserToolResponse {
                    tool_name,
                    summary: user_response_summary.clone(),
                    data: user_response_data,
                    icon: Some("🌐".to_string()),
                },
            ))
        }
        Err(e) => Err(format!("Narrativ browse_raw failed: {e}")),
    }
}

#[cfg(test)]
mod tests {
    // Basic test structure.
    #[tokio::test]
    async fn test_handle_narrativ_browse_raw_placeholder() {
        // This is a placeholder. A full test would require mocking the zyte API call.
        assert!(true);
    }
}