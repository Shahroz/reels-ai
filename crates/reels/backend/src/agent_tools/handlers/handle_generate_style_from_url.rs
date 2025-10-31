//! Handles the 'generate_style_from_url' agent tool action.
//!
//! This function takes `GenerateStyleFromUrlParams`, adapts the logic from
//! the `create_style` route to fetch HTML from a URL, generate assets,
//! and create a new style record in the database.

use base64::Engine;

use crate::queries::user_credit_allocation::{deduct_user_credits_with_transaction, CreditChangesParams};
use bigdecimal::BigDecimal;

// Function length justification:
// This function orchestrates several I/O-heavy operations: fetching content from a URL via Zyte,
// uploading multiple assets (HTML, screenshot) to GCS, and database insertion.
// Breaking it down further would obscure the sequential, single-purpose nature of the task.
#[allow(clippy::too_many_lines)]
pub async fn handle_generate_style_from_url(
    pool: &sqlx::PgPool,
    gcs: &dyn crate::services::gcs::gcs_operations::GCSOperations,
    screenshot_service: &dyn crate::services::screenshot::screenshot_service::ScreenshotService,
    params: crate::agent_tools::tool_params::generate_style_from_url_params::GenerateStyleFromUrlParams,
    _user_id: uuid::Uuid, // Passed for compatibility with dispatch, but extracted from params
) -> std::result::Result<(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse), std::string::String> {
    let user_id = params.user_id.ok_or("The user_id should be provided".to_owned())?;
    let organization_id = params.organization_id;
    let style_name = params.name.clone();
    let source_url = params.source_url.clone();

    // Check credit availability before proceeding
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::GENERATE_STYLE;
    
    if let Err(error) = crate::services::credits_service::check_credits_availability_by_user_or_organization(
        pool,
        user_id,
        credits_to_consume,
        organization_id,
    ).await {
        return std::result::Result::Err(error.message);
    }

    log::info!(
        "Fetching HTML content for style '{style_name}' from URL: {source_url}"
    );

    let zyte_api_key = match std::env::var("ZYTE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            log::error!("ZYTE_API_KEY environment variable not set.");
            return std::result::Result::Err("Server configuration error: Missing API key.".to_string());
        }
    };
    let zyte_client = crate::zyte::zyte::ZyteClient::new(zyte_api_key);

    let mut final_html_content = match zyte_client.extract_styles_with_fallback(&source_url).await {
        Ok(fetched_html) => {
            log::info!("Successfully fetched HTML from {source_url}");
            fetched_html
        }
        Err(e) => {
            log::error!("Failed to fetch HTML from URL {source_url}: {e}");
            return std::result::Result::Err(format!("Failed to fetch style from source URL: {e}"));
        }
    };

    let bucket = match std::env::var("GCS_BUCKET") {
        Ok(b) => b,
        Err(_) => {
            log::error!("GCS_BUCKET environment variable not set.");
            return std::result::Result::Err("Server configuration error: Missing GCS_BUCKET.".to_string());
        }
    };
    let style_id = uuid::Uuid::new_v4();

    let processed_html_result = crate::utils::minimize_large_html_content::minimize_large_html_content(
        &final_html_content,
        gcs,
        &bucket,
        style_id,
        300_000,
        1000,
    )
    .await;

    final_html_content = match processed_html_result {
        Ok(processed_html) => processed_html,
        Err(e) => {
            log::error!("Error processing data URIs for style {style_id}: {e}");
            return std::result::Result::Err(format!("Failed to process style content: {e}"));
        }
    };

    let html_object = format!("styles/{style_id}/style.html");
    let html_bytes = final_html_content.clone().into_bytes();
    let html_gcs_url = match gcs.upload_raw_bytes(&bucket, &html_object, "text/html", html_bytes, true, crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic).await {
        Ok(url) => url,
        Err(e) => {
            log::error!("Failed to upload style HTML to GCS: {e:#}");
            return std::result::Result::Err("Failed to store style HTML.".to_string());
        }
    };
    let html_url = crate::services::gcs::convert_to_pages_url::convert_to_pages_url(&html_gcs_url);

    let screenshot_base64 = match screenshot_service.screenshot_website(&html_url, true).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to screenshot style HTML: {e}");
            return std::result::Result::Err("Failed to screenshot style.".to_string());
        }
    };
    let screenshot_data = match base64::engine::general_purpose::STANDARD.decode(&screenshot_base64) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::error!("Invalid base64 in screenshot data: {e}");
            return std::result::Result::Err("Failed to process screenshot data.".to_string());
        }
    };

    let screenshot_object = format!("styles/{style_id}/screenshot.png");
    let screenshot_gcs_url = match gcs.upload_raw_bytes(&bucket, &screenshot_object, "image/png", screenshot_data, false, crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic).await {
        Ok(url) => url,
        Err(e) => {
            log::error!("Failed to upload screenshot to GCS: {e:#}");
            return std::result::Result::Err("Failed to store style screenshot.".to_string());
        }
    };
    let screenshot_url = crate::services::gcs::convert_to_pages_url::convert_to_pages_url(&screenshot_gcs_url);

    let result = sqlx::query_as!(
        crate::db::styles::Style,
        r#"
        INSERT INTO styles (id, user_id, name, html_url, screenshot_url, is_public)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, name, html_url, screenshot_url, is_public, created_at, updated_at
        "#,
        style_id,
        Some(user_id),
        style_name,
        html_url.clone(),
        screenshot_url.clone(),
        false // is_public = false for private styles
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(style) => {
            // Consume credits before returning response
            let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::GENERATE_STYLE;
            let tool_name = "generate_style_from_url".to_string();
            let deduction_params = CreditChangesParams {
                user_id,
                organization_id, // Use organization_id from params
                credits_to_change: BigDecimal::from(credits_to_consume),
                action_source: "agent_tool".to_string(),
                action_type: tool_name.to_string(),
                entity_id: Some(style.id.clone()),
            };
            if let Err(e) = deduct_user_credits_with_transaction(pool, deduction_params).await {
                log::error!("Failed to deduct {} credits for user {} after generate style from url: {}", credits_to_consume, user_id, e);
            }

            let full_response =
                agentloop::types::full_tool_response::FullToolResponse {
                    tool_name: tool_name.clone(),
                    response: serde_json::to_value(&style).map_err(|e| {
                        format!("Failed to serialize style response: {e}")
                    })?,
                };
            let user_response =
                agentloop::types::user_tool_response::UserToolResponse {
                    tool_name: tool_name.clone(),
                    summary: format!(
                        "Successfully generated style '{}' (ID: {}) from URL.",
                        style.name, style.id
                    ),
                    icon: None,
                    data: Some(full_response.response.clone()),
                };
            std::result::Result::Ok((full_response, user_response))
        }
        Err(e) => {
            log::error!("Error creating style for user {user_id}: {e}");
            std::result::Result::Err("Failed to create style in database".to_string())
        }
    }
}
