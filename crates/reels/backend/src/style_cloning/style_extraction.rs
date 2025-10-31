use std::time::Instant;
use anyhow::Result;
use sqlx::PgPool;
use url::Url;
use tracing::instrument;

// Import DB request functions
use crate::db::requests;
use crate::db::requests::{CreateRequestArgs, UpdateRequestArgs};

// Import Zyte client
use crate::integrations::zyte::ZyteClient;
// Import screenshot_html for visual finetuning
use crate::library_docs::website_style/visual_html_finetuning::screenshot_html;
// Import improve_html_style for visual feedback refinement
use crate::library_docs::website_style/visual_html_finetuning::html_improvement::improve_html_style;

/// Parameters for style extraction
#[derive(Debug, Clone)]
pub struct ExtractStyleParams {
    pub target_url: Url,
    pub content_to_style: Option<String>,
    pub what_to_create: Option<String>,
    pub user_id: Option<sqlx::types::Uuid>,
    pub visual_feedback_enabled: bool,
    pub max_tries: Option<usize>,
    pub min_score: Option<usize>,
    pub db_pool: PgPool,
    pub zyte_api_key: String,
}

/// Output of style extraction
#[derive(Debug, Clone)]
pub struct StyleExtractionOutput {
    pub request_id: i32,
    pub final_description: String,
    pub compressed_output_html: Option<Vec<u8>>,
}

/// Extracts style from a given URL and persists request details in the database.
/// This function uses the Zyte client to extract styled HTML and optionally refines it if visual feedback is enabled.
#[instrument(skip(params))]
pub async fn extract_style(params: ExtractStyleParams) -> Result<StyleExtractionOutput> {
    let start_time = Instant::now();

    // Create an initial request record in the database
    let create_args = CreateRequestArgs {
        url: Some(params.target_url.to_string()),
        content_to_style: params.content_to_style.clone(),
        what_to_create: params.what_to_create.clone(),
        status: "Initializing".to_string(),
        user_id: params.user_id,
        visual_feedback: Some(params.visual_feedback_enabled),
    };
    let request_id = requests::create_request(&params.db_pool, create_args).await?;
    requests::update_request_status(&params.db_pool, request_id, "Processing").await?;

    // Instantiate Zyte client
    let zyte_client = ZyteClient::new(params.zyte_api_key.clone());

    // Extract styled HTML using the Zyte client
    let initial_html = match params.content_to_style.as_ref() {
        Some(content) => {
            zyte_client.extract_and_apply_styles(params.target_url.as_str(), content, false).await?
        },
        None => {
            zyte_client.extract_and_apply_styles(params.target_url.as_str(), "", false).await?
        }
    };

    let mut final_html = initial_html.clone();

    // Optionally refine the HTML if visual feedback is enabled
    if params.visual_feedback_enabled {
        let max_tries = params.max_tries.unwrap_or(5);
        let min_score = params.min_score.unwrap_or(80);
        log::info!("Visual feedback enabled for request {}. Starting refinement loop.", request_id);
        final_html = improve_html_style(&final_html, max_tries, min_score, None).await;
        log::info!("Refinement loop completed. Final HTML size: {} bytes", final_html.len());
    } else {
        log::info!("Visual feedback disabled for request {}. Skipping refinement loop.", request_id);
    }

    // Generate a summary description (placeholder)
    let final_description = format!("Extracted style from {}. Generated HTML size: {} bytes.", params.target_url, final_html.len());

    let elapsed_ms = start_time.elapsed().as_millis() as i32;

    // Update the request record with the final status and output details
    let update_args = UpdateRequestArgs {
        compressed_style_website_content: None,
        compressed_output_html: Some(final_html.clone().into_bytes()),
        status: "Completed".to_string(),
        execution_time_ms: Some(elapsed_ms),
        credits_used: Some(1),
    };
    requests::update_request_completion(&params.db_pool, request_id, update_args).await?;

    Ok(StyleExtractionOutput {
        request_id,
        final_description,
        compressed_output_html: Some(final_html.into_bytes()),
    })
}
