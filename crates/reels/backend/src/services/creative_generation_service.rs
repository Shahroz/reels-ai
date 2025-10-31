//! Provides a function to process and generate a single creative for a specific format.
//!
//! This function encapsulates the logic for taking style, assets, document context,
//! and a specific creative format, then using an LLM to generate HTML content,
//! storing it, generating a screenshot, and saving the creative record to the database.

use crate::db::creatives::Creative;
use crate::queries::user_credit_allocation::{deduct_user_credits_with_transaction, CreditChangesParams};
use bigdecimal::BigDecimal;
use crate::routes::creatives::creative_asset_utils::upload_creative_assets;
use crate::routes::creatives::generate_creative::CombinedFormatInfo;
use crate::routes::creatives::responses::CreativeResponse;
use actix_web::web;
use llm::llm_typed_unified::vendor_model::VendorModel;
use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use serde_json;
use log;
use tokio;
use llm::llm_typed_unified::llm::llm;

// Function length justification:
// This function orchestrates several potentially long-running operations:
// LLM interaction (with retries), GCS uploads, and database interaction.
// While it exceeds the typical 50 LoC guideline, breaking it down further
// would obscure the sequential nature of generating a single creative for one format.
// The core logic forms a cohesive unit.
#[allow(clippy::too_many_arguments)] // Justified by the need to pass diverse context for generation
pub async fn process_single_creative_format_for_generation(
    pool_data: web::Data<PgPool>,
    gcs_data: web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
    style_id: Uuid,
    style_name: String,
    style_html_content: String,
    assets_context_str: String,
    document_context_str: String,
    current_format_info_owned: CombinedFormatInfo,
    payload_collection_id_val: Uuid, // Non-optional as per signature
    db_asset_ids: Option<Vec<Uuid>>,
    db_document_ids: Option<Vec<Uuid>>,
    llm_models_arc: Arc<Vec<VendorModel>>,
    max_attempts: u32,
    creative_name: String, // Add name parameter
    user_id: Uuid, // Add user_id parameter
    organization_id: Option<Uuid>, // Add organization_id parameter
) -> std::result::Result<CreativeResponse, String> {
    let pool_ref = pool_data.get_ref();
    let gcs_client_ref = gcs_data.get_ref().as_ref();

    // Construct creative_format_context specifically for the current format
    let mut current_format_specific_context_str = String::new();
    let mut single_format_context_part = format!(
        "Name: {}, Description: {}, Dimensions: {}x{}",
        current_format_info_owned.name,
        current_format_info_owned.description.as_deref().unwrap_or("N/A"),
        current_format_info_owned.width.unwrap_or(0),
        current_format_info_owned.height.unwrap_or(0)
    );
    if let Some(meta) = &current_format_info_owned.metadata {
        single_format_context_part.push_str(&format!(
            "\nMetadata: {}",
            serde_json::to_string_pretty(meta).unwrap_or_else(|_| meta.to_string())
        ));
    }
    current_format_specific_context_str.push_str(&format!(
        "\n---\nFormat ID: {}\n{}\n---\n",
        current_format_info_owned.id, single_format_context_part
    ));

    // Construct LLM Prompt for the current format
    let prompt = format!(
        r#"Generate a complete, self-contained HTML creative based on the provided context.
The final output must be ONLY the raw HTML code, starting with <!DOCTYPE html> or <html> and ending with </html>.
Include all necessary CSS and JavaScript derived from the STYLE directly within the HTML (e.g., in <style> tags or inline styles).
Use the provided ASSET URLs for images or other resources.

CONTEXT:

<STYLE name="{}">
{}
</STYLE>

<ASSETS>
{}
</ASSETS>
{}
{}
TASK: Create the HTML output by following these instructions:
1.  **Style Guidance:** Use the provided `<STYLE>` block as the primary reference for stylistic choices. This includes color palettes, typography, layout principles, and any specific HTML components or CSS classes defined within the style's HTML content.
2.  **Asset Integration:** Incorporate the assets listed in `<ASSETS>` into the HTML structure appropriately. Use the provided URLs directly.
3.  **Content Foundation:** Base the textual and informational content of the creative primarily on the information provided in the `<DOCUMENT_CONTEXTS>` section, if present.
4.  **Format Adherence:** Ensure the final HTML structure and dimensions align with the requirements outlined in the `<CREATIVE_FORMAT_CONTEXTS>`. Pay attention to the specified name, description, dimensions (width/height), and any metadata hints for the specific format ID {}.
5.  **Output Requirements:** Generate only the raw HTML code, starting with `<!DOCTYPE html>` or `<html>` and ending with `</html>`. Embed all necessary CSS and JavaScript within the HTML document (e.g., in `<style>` tags or inline styles derived from the STYLE context). Do not include any explanatory text or markdown formatting around the HTML code itself.

Create the HTML output"#,
        style_name,
        style_html_content,
        assets_context_str,
        if document_context_str.is_empty() {
            "".to_string()
        } else {
            format!("\n<DOCUMENT_CONTEXTS>\n{document_context_str}\n</DOCUMENT_CONTEXTS>")
        },
        format!(
            "\n<CREATIVE_FORMAT_CONTEXTS>\n{}\n</CREATIVE_FORMAT_CONTEXTS>",
            current_format_specific_context_str
        ),
        current_format_info_owned.id
    );

    log::info!("Sending prompt length: {} for format ID: {}", prompt.len(), current_format_info_owned.id);

    let mut llm_attempt_for_current_format = 0;
    loop {
        llm_attempt_for_current_format += 1;
        if llm_models_arc.is_empty() {
            return Err("LLM model list is empty.".to_string());
        }
        let model_idx = (llm_attempt_for_current_format as usize - 1) % llm_models_arc.len();
        let model_to_use = llm_models_arc[model_idx].clone();

        log::info!(
            "Attempt {}/{} for format ID {} using model {:?}",
            llm_attempt_for_current_format,
            max_attempts,
            current_format_info_owned.id,
            model_to_use
        );

        let llm_result = llm(
            false,
            &prompt,
            vec![model_to_use],
            1, // unified_llm internal retries
        )
        .await;

        match llm_result {
            Ok(html_content) => {
                let trimmed_content = html_content.trim().trim_start_matches("```html").trim_end_matches("```").to_string();
                let is_long_enough = trimmed_content.len() >= 2000; // Validation criteria

                if is_long_enough {
                    let creative_id = Uuid::new_v4();
                    let html_content_bytes = trimmed_content.into_bytes();

                    let (html_url, screenshot_url) =
                        match upload_creative_assets(gcs_client_ref, creative_id, html_content_bytes).await {
                        Ok(urls) => urls,
                        Err(e) => {
                            log::error!("GCS upload failed for creative {creative_id}: {e}");
                            return Err(format!("GCS upload failed: {e}"));
                        }
                    };

                    #[derive(sqlx::FromRow, Debug)]
                    struct NewCreativeDetails {
                        id: Uuid,
                        name: String,
                        collection_id: Option<Uuid>,
                        creative_format_id: Uuid,
                        style_id: Option<Uuid>,
                        document_ids: Option<Vec<Uuid>>,
                        asset_ids: Option<Vec<Uuid>>,
                        html_url: String,
                        draft_url: Option<String>,
                        bundle_id: Option<Uuid>,
                        screenshot_url: String,
                        is_published: bool,
                        publish_url: Option<String>,
                        created_at: chrono::DateTime<chrono::Utc>,
                        updated_at: chrono::DateTime<chrono::Utc>,
                        creator_email: Option<String>,
                        current_user_access_level: Option<String>,
                    }

                    let asset_ids_slice = db_asset_ids.as_deref();
                    let document_ids_slice = db_document_ids.as_deref();

                    let insert_result = sqlx::query_as!(
                        NewCreativeDetails,
                        r#"
                        INSERT INTO creatives (
                            id, name, collection_id, creative_format_id, style_id, document_ids,
                            asset_ids, html_url, bundle_id, screenshot_url, is_published, publish_url,
                            created_at, updated_at
                        )
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
                        RETURNING
                            id, name, collection_id, creative_format_id, style_id, document_ids,
                            asset_ids, html_url, draft_url, bundle_id, screenshot_url, is_published, publish_url,
                            created_at, updated_at,
                            (SELECT u.email FROM users u JOIN collections col ON u.id = col.user_id WHERE col.id = $3) AS creator_email,
                            'owner'::text AS current_user_access_level
                        "#,
                        creative_id,
                        creative_name,
                        Some(payload_collection_id_val), // This is $3, the collection_id for the subquery in RETURNING
                        current_format_info_owned.id,
                        style_id,
                        document_ids_slice,
                        asset_ids_slice,
                        html_url,
                        None::<Uuid>, // bundle_id
                        screenshot_url,
                        false,        // is_published
                        None::<String> // publish_url
                    )
                    .fetch_one(pool_ref)
                    .await;

                    match insert_result {
                        Ok(details) => {
                            log::info!("Successfully generated and saved creative {} for format {}", details.id, current_format_info_owned.id);
                            // Consume credits before returning response
                            let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::GENERATE_CREATIVE;
                            let deduction_params = CreditChangesParams {
                                user_id,
                                organization_id, // Use the passed organization_id
                                credits_to_change: BigDecimal::from(credits_to_consume),
                                action_source: "api".to_string(),
                                action_type: "generate_creative".to_string(),
                                entity_id: Some(details.id),
                            };
                            if let Err(e) = deduct_user_credits_with_transaction(pool_ref, deduction_params).await {
                                log::error!("Failed to deduct {} credits for user {} after generating creative: {}", credits_to_consume, user_id, e);
                            }
                            let response = CreativeResponse {
                                creative: Creative {
                                    id: details.id,
                                    name: details.name,
                                    collection_id: details.collection_id,
                                    creative_format_id: details.creative_format_id,
                                    style_id: details.style_id,
                                    document_ids: details.document_ids,
                                    asset_ids: details.asset_ids,
                                    html_url: details.html_url,
                                    draft_url: details.draft_url,
                                    bundle_id: details.bundle_id,
                                    screenshot_url: details.screenshot_url,
                                    is_published: details.is_published,
                                    publish_url: details.publish_url,
                                    created_at: details.created_at,
                                    updated_at: details.updated_at,
                                },
                                creator_email: details.creator_email,
                                current_user_access_level: details.current_user_access_level,
                            };
                            return Ok(response);
                        }
                        Err(e) => {
                            log::error!("DB error saving creative for format {}: {:?}", current_format_info_owned.id, e);
                            // If DB error occurs, it's unlikely to be fixed by retrying LLM.
                            // However, if it's a transient DB issue, retrying might be desired by outer logic.
                            // For now, if DB fails, we treat it as a hard failure for this attempt.
                            if llm_attempt_for_current_format >= max_attempts {
                                return Err(format!("DB error saving creative after max LLM retries: {e}"));
                            }
                            // Continue to next LLM attempt if DB error, assuming it might be related to bad LLM output causing DB constraint violation.
                        }
                    }
                } else {
                    log::warn!(
                        "LLM output validation failed for format ID {} on attempt {}/{}. Length >= 2000: {}. Response head: {:?}",
                        current_format_info_owned.id,
                        llm_attempt_for_current_format,
                        max_attempts,
                        is_long_enough,
                        trimmed_content.chars().take(100).collect::<String>()
                    );
                    if llm_attempt_for_current_format >= max_attempts {
                        return Err(format!("LLM output validation failed after {} attempts for format {}.", max_attempts, current_format_info_owned.id));
                    }
                }
            }
            Err(e) => {
                log::error!(
                    "LLM generation call failed for format ID {} on attempt {}/{}: {:?}",
                    current_format_info_owned.id,
                    llm_attempt_for_current_format,
                    max_attempts,
                    e
                );
                if llm_attempt_for_current_format >= max_attempts {
                    return Err(format!("LLM generation failed after {} attempts for format {}: {}", max_attempts, current_format_info_owned.id, e));
                }
            }
        }

        // If loop continues, it means a retry is happening
        log::info!(
            "Retrying LLM call for format {}, attempt {} of {}.",
            current_format_info_owned.id,
            llm_attempt_for_current_format + 1, // Next attempt number
            max_attempts
        );
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Small delay before retrying
    }

    // Should not be reached if logic is correct, as loop either returns Ok or Err after max_attempts.
    // However, as a fallback:
    // Err(format!("Failed to generate creative for format {} after {} attempts.", current_format_info_owned.id, max_attempts))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert!(true, "Placeholder test for process_single_creative_format_for_generation");
        // Comprehensive tests would require mocking GCSClient, PgPool, LLM calls,
        // and setting up CombinedFormatInfo and other parameters.
    }
}