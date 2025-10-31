//! Implements the logic for the Narrativ browse_with_query tool.
//!
//! This function takes a URL and a query, fetches content, uses an LLM
//! to process the content based on the query, and returns a summary.
//! Adheres to Narrativ and AgentLoop coding standards.

// Note: Using fully qualified paths as per guidelines where applicable.

use llm::llm_typed_unified::llm::llm;
use llm::llm_typed_unified::vendor_model::VendorModel;
use llm::vendors::gemini::gemini_model::GeminiModel;

use crate::queries::user_credit_allocation::{deduct_user_credits_with_transaction, CreditChangesParams};
use bigdecimal::BigDecimal;

pub async fn handle_narrativ_browse_with_query(
    params: crate::agent_tools::tool_params::browse_with_query_params::BrowseWithQueryParams,
    user_id: uuid::Uuid,
    pool: &sqlx::PgPool,
) -> Result<(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse), String> {
    let tool_name = "narrativ_browse_with_query".to_string();
    
    // Check credit availability before proceeding
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::NARRATIV_BROWSE_WITH_QUERY;
    
    if let Err(error) = crate::services::credits_service::check_credits_availability_by_user_or_organization(
        pool,
        user_id,
        credits_to_consume,
        None, // organization_id not available in params yet
    ).await {
        return std::result::Result::Err(error.message);
    }
    
    match api_tools::zyte::fetch_browser_html::fetch_browser_html_and_extract_text(&params.url).await {
        Ok(text_extraction) => {
            const MAX_CONTENT_LEN: usize = 1_000_000; // As per instruction trimming to 1M tokens (using chars as proxy)
            let mut content = text_extraction.content.clone();
            if content.len() > MAX_CONTENT_LEN {
                content.truncate(MAX_CONTENT_LEN);
            }

            let prompt = format!(
                "<WEBSITE_CONTENT>\n{}\n</WEBSITE_CONTENT>\n<QUERY>\n{}\n</QUERY>\nTASK:\nBased on the WEBSITE_CONTENT, answer the QUERY. Summarize the content and extract key information relevant to the query. Return only the answer, without any preamble. As part of the response if the content does not answer the query please extract links that can serve as continuation of the reseaarch",
                content, &params.query
            );

            let models_to_try: std::vec::Vec<VendorModel> =
                vec![VendorModel::Gemini(GeminiModel::Gemini25Flash)];

            let (user_response_summary, llm_summary) = match llm(false, &prompt, models_to_try, 3).await {
                Ok(summary) => {
                    let user_summary = format!(
                        "Browsing for '{}' and processing query '{}' completed.",
                        params.url, &params.query
                    );
                    (user_summary, summary)
                }
                Err(e) => {
                    tracing::error!(
                        "LLM summarization failed for browse query '{}' on url '{}': {}",
                        &params.query,
                        params.url,
                        e
                    );
                    return Err(format!("LLM summarization failed: {e}"));
                }
            };

            let mut full_response_map = serde_json::Map::new();
            full_response_map.insert("status".to_string(), serde_json::json!("success"));
            full_response_map
                .insert("content".to_string(), serde_json::json!(text_extraction.content));
            full_response_map.insert("summary".to_string(), serde_json::json!(llm_summary.clone()));
            
            let user_response_data = Some(serde_json::json!({ "summary": llm_summary }));

            let full_response_properties = serde_json::Value::Object(full_response_map);

            // Consume credits before returning response
            let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::NARRATIV_BROWSE_WITH_QUERY;
            let deduction_params = CreditChangesParams {
                user_id,
                organization_id: None, // TODO: Get from user context if available
                credits_to_change: BigDecimal::from(credits_to_consume),
                action_source: "agent_tool".to_string(),
                action_type: "narrativ_browse_with_query".to_string(),
                entity_id: None, // No specific entity for browsing
            };
            if let Err(e) = deduct_user_credits_with_transaction(pool, deduction_params).await {
                log::error!("Failed to deduct {} credits for user {} after narrativ browse with query: {}", credits_to_consume, user_id, e);
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
        Err(e) => Err(format!("Narrativ browse_with_query failed: {e}")),
    }
}

#[cfg(test)]
mod tests {
    // Basic test structure.
    #[tokio::test]
    async fn test_handle_narrativ_browse_with_query_placeholder() {
        // This is a placeholder. A full test would require mocking the zyte API and LLM calls.
        assert!(true);
    }
}